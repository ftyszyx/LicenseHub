use crate::apis::system_settings_handler::get_resource_storage_channel_id;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use data_model::{resources, storage_channels};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter,
    QueryOrder, Set,
};
use std::path::Path;
use storage_adapter::{
    DeleteRequest, DownloadRequest, DownloadResult, PROVIDER_MOCK, StorageChannelConfig,
    UploadRequest, default_registry, parse_storage_config,
};
use uuid::Uuid;

pub const MAX_RESOURCE_SIZE: u64 = 20 * 1024 * 1024;

#[derive(Debug)]
pub struct ResourceUpload {
    pub resource_type: String,
    pub original_name: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

pub fn normalize_resource_type(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_ascii_lowercase();
    if value.is_empty()
        || value.len() > 64
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(AppError::validation(
            "resource_type must contain only letters, numbers, '_' or '-'",
        ));
    }
    Ok(value)
}

pub async fn upload_resource(
    state: &AppState,
    db: &DatabaseTransaction,
    uploaded_by: i32,
    input: ResourceUpload,
) -> Result<resources::Model, AppError> {
    let resource_type = normalize_resource_type(&input.resource_type)?;
    if input.content.is_empty() || input.content.len() as u64 > MAX_RESOURCE_SIZE {
        return Err(AppError::validation(
            "resource must be between 1 byte and 20 MB",
        ));
    }
    let content_type = input.content_type.trim().to_ascii_lowercase();
    if content_type.is_empty() || content_type.len() > 128 {
        return Err(AppError::validation("resource content type is invalid"));
    }
    let original_name = Path::new(&input.original_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("resource")
        .chars()
        .take(255)
        .collect::<String>();
    let channel = select_storage_channel(state, db).await?;
    let config: StorageChannelConfig = parse_storage_config(&channel.provider, &channel.config)?;
    let extension = Path::new(&original_name)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("bin");
    let object_key = format!(
        "{}/resources/{}/{}.{}",
        config.prefix,
        resource_type,
        Uuid::new_v4().simple(),
        extension
    );
    let registry = default_registry();
    let provider = if config.endpoint.starts_with("mock://") {
        PROVIDER_MOCK
    } else {
        channel.provider.as_str()
    };
    let adapter = registry.get(provider)?;
    adapter
        .upload(UploadRequest {
            config: &config,
            object_key: &object_key,
            body: &input.content,
            content_type: &content_type,
            private: true,
        })
        .await
        .map_err(AppError::from)?;

    let now = chrono::Utc::now().fixed_offset();
    Ok(resources::ActiveModel {
        storage_channel_id: Set(channel.id),
        object_key: Set(object_key),
        resource_type: Set(resource_type),
        original_name: Set(original_name),
        content_type: Set(content_type),
        size: Set(input.content.len() as i64),
        uploaded_by: Set(uploaded_by),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn download_resource(
    state: &AppState,
    resource: &resources::Model,
) -> Result<DownloadResult, AppError> {
    let channel = storage_channels::Entity::find_by_id(resource.storage_channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::not_found("storage_channels", Some(resource.storage_channel_id))
        })?;
    let config = parse_storage_config(&channel.provider, &channel.config)?;
    let registry = default_registry();
    let provider = if config.endpoint.starts_with("mock://") {
        PROVIDER_MOCK
    } else {
        channel.provider.as_str()
    };
    registry
        .get(provider)?
        .download(DownloadRequest {
            config: &config,
            object_key: &resource.object_key,
        })
        .await
        .map_err(AppError::from)
}

pub async fn delete_resource(state: &AppState, resource: resources::Model) -> Result<(), AppError> {
    let channel = storage_channels::Entity::find_by_id(resource.storage_channel_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::not_found("storage_channels", Some(resource.storage_channel_id))
        })?;
    let config = parse_storage_config(&channel.provider, &channel.config)?;
    let registry = default_registry();
    let provider = if config.endpoint.starts_with("mock://") {
        PROVIDER_MOCK
    } else {
        channel.provider.as_str()
    };
    registry
        .get(provider)?
        .delete(DeleteRequest {
            config: &config,
            object_key: &resource.object_key,
        })
        .await
        .map_err(AppError::from)?;
    resource.into_active_model().delete(&state.db).await?;
    Ok(())
}

async fn select_storage_channel(
    state: &AppState,
    db: &DatabaseTransaction,
) -> Result<storage_channels::Model, AppError> {
    let configured_id = get_resource_storage_channel_id(state).await?;
    if configured_id > 0 {
        return storage_channels::Entity::find_by_id(configured_id)
            .filter(storage_channels::Column::Status.eq(1_i16))
            .one(db)
            .await?
            .ok_or_else(|| {
                AppError::validation("configured resource storage channel is not enabled")
            });
    }
    storage_channels::Entity::find()
        .filter(storage_channels::Column::Status.eq(1_i16))
        .order_by_asc(storage_channels::Column::SortOrder)
        .order_by_asc(storage_channels::Column::Id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::validation("no enabled resource storage channel is available"))
}
