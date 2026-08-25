use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::apis::system_settings_handler::get_resource_storage_channel_id;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::{DateTime, SecondsFormat, Utc};
use data_model::{app_version_sync_logs, apps, storage_channels};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use storage_adapter::{
    PROVIDER_MOCK, StorageChannelConfig, StorageError, UploadRequest, UploadResult,
    build_object_key, build_public_url, default_registry, normalize_storage_provider,
    parse_storage_config,
};

const LOG_STATUS_PENDING: i16 = 0;
const LOG_STATUS_SUCCESS: i16 = 1;
const LOG_STATUS_FAILED: i16 = 2;

impl From<StorageError> for AppError {
    fn from(error: StorageError) -> Self {
        match error {
            StorageError::Config(message) => AppError::validation(message),
            StorageError::Unsupported(provider) => {
                AppError::validation(format!("storage provider is not supported: {provider}"))
            }
            StorageError::Request(_) | StorageError::Response(_) | StorageError::Signature(_) => {
                AppError::ExternalService {
                    service: "storage".to_string(),
                    error: error.to_string(),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum StorageChannelStatus {
    Disabled = 0,
    Enabled = 1,
}

impl From<i16> for StorageChannelStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => StorageChannelStatus::Enabled,
            _ => StorageChannelStatus::Disabled,
        }
    }
}

impl From<StorageChannelStatus> for i16 {
    fn from(value: StorageChannelStatus) -> Self {
        value as i16
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateStorageChannelReq {
    pub name: String,
    pub provider: String,
    pub status: StorageChannelStatus,
    pub sort_order: Option<i32>,
    pub config: Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStorageChannelReq {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub status: Option<StorageChannelStatus>,
    pub sort_order: Option<i32>,
    pub config: Option<Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListStorageChannelsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub id: Option<i32>,
    pub provider: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct StorageChannelInfo {
    pub id: i32,
    pub name: String,
    pub provider: String,
    pub status: StorageChannelStatus,
    pub sort_order: i32,
    pub config: Value,
    pub created_at: DateTime<chrono::FixedOffset>,
    pub updated_at: DateTime<chrono::FixedOffset>,
}

impl TryFrom<storage_channels::Model> for StorageChannelInfo {
    type Error = AppError;

    fn try_from(channel: storage_channels::Model) -> Result<Self, Self::Error> {
        let config = response_storage_config(&channel.provider, &channel.config)?;
        Ok(Self {
            id: channel.id,
            name: channel.name,
            provider: channel.provider,
            status: StorageChannelStatus::from(channel.status),
            sort_order: channel.sort_order,
            config,
            created_at: channel.created_at,
            updated_at: channel.updated_at,
        })
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct SyncVersionReq {
    pub channel_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize)]
pub struct SyncVersionResponse {
    pub app_id: i32,
    pub results: Vec<SyncVersionResult>,
}

#[derive(Debug, Serialize)]
pub struct SyncVersionResult {
    pub channel_id: i32,
    pub channel_name: String,
    pub provider: String,
    pub object_key: String,
    pub public_url: String,
    pub success: bool,
    pub status: i16,
    pub etag: Option<String>,
    pub error_message: Option<String>,
    pub log_id: i64,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListVersionSyncLogsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub app_id: Option<i32>,
    pub storage_channel_id: Option<i32>,
    pub provider: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct VersionSyncLogInfo {
    pub id: i64,
    pub app_id: i32,
    pub storage_channel_id: i32,
    pub provider: String,
    pub object_key: String,
    pub public_url: String,
    pub manifest: Value,
    pub status: i16,
    pub error_message: Option<String>,
    pub etag: Option<String>,
    pub created_at: DateTime<chrono::FixedOffset>,
    pub finished_at: Option<DateTime<chrono::FixedOffset>>,
}

impl From<app_version_sync_logs::Model> for VersionSyncLogInfo {
    fn from(log: app_version_sync_logs::Model) -> Self {
        Self {
            id: log.id,
            app_id: log.app_id,
            storage_channel_id: log.storage_channel_id,
            provider: log.provider,
            object_key: log.object_key,
            public_url: log.public_url,
            manifest: log.manifest,
            status: log.status,
            error_message: log.error_message,
            etag: log.etag,
            created_at: log.created_at,
            finished_at: log.finished_at,
        }
    }
}

#[handler]
pub async fn create_storage_channel(
    depot: &mut Depot,
    req: JsonBody<CreateStorageChannelReq>,
) -> Result<ApiResponse<StorageChannelInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let channel = create_storage_channel_impl(state, req.into_inner()).await?;
    Ok(ApiResponse::success(channel))
}

pub async fn create_storage_channel_impl(
    state: &AppState,
    req: CreateStorageChannelReq,
) -> Result<StorageChannelInfo, AppError> {
    let provider = normalize_storage_provider(&req.provider)?;
    let name = normalize_required_text(req.name, "name")?;
    let config = normalize_storage_config(&provider, &req.config)?;
    let now = Utc::now().fixed_offset();
    let active = storage_channels::ActiveModel {
        name: Set(name),
        provider: Set(provider),
        status: Set(i16::from(req.status)),
        sort_order: Set(req.sort_order.unwrap_or_default()),
        config: Set(config),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let channel = active.insert(&state.db).await?;
    get_storage_channel_by_id_impl(state, channel.id).await
}

#[handler]
pub async fn update_storage_channel(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UpdateStorageChannelReq>,
) -> Result<ApiResponse<StorageChannelInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let channel = update_storage_channel_impl(state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(channel))
}

pub async fn update_storage_channel_impl(
    state: &AppState,
    id: i32,
    req: UpdateStorageChannelReq,
) -> Result<StorageChannelInfo, AppError> {
    let channel = storage_channels::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("storage_channels", Some(id)))?;

    let final_provider = match req.provider.as_deref() {
        Some(provider) => normalize_storage_provider(provider)?,
        None => channel.provider.clone(),
    };
    let final_config = match req.config.as_ref() {
        Some(config) => normalize_storage_config(&final_provider, config)?,
        None => normalize_storage_config(&final_provider, &channel.config)?,
    };

    let resource_storage_channel_id = get_resource_storage_channel_id(state).await?;
    if resource_storage_channel_id == id
        && req
            .status
            .is_some_and(|status| status == StorageChannelStatus::Disabled)
    {
        return Err(AppError::validation(
            "change the resource storage channel before disabling this channel",
        ));
    }

    let mut active = channel.into_active_model();
    if let Some(name) = req.name {
        active.name = Set(normalize_required_text(name, "name")?);
    }
    active.provider = Set(final_provider);
    if let Some(status) = req.status {
        active.status = Set(i16::from(status));
    }
    if let Some(sort_order) = req.sort_order {
        active.sort_order = Set(sort_order);
    }
    active.config = Set(final_config);
    active.updated_at = Set(Utc::now().fixed_offset());
    let updated = active.update(&state.db).await?;
    get_storage_channel_by_id_impl(state, updated.id).await
}

#[handler]
pub async fn delete_storage_channel(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let id = id.into_inner();
    let channel = storage_channels::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("storage_channels", Some(id)))?;
    if get_resource_storage_channel_id(state).await? == id {
        return Err(AppError::validation(
            "change the resource storage channel before deleting this channel",
        ));
    }
    channel.into_active_model().delete(&state.db).await?;
    Ok(ApiResponse::success(()))
}

#[handler]
pub async fn list_storage_channels(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<StorageChannelInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListStorageChannelsParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = storage_channels::Entity::find()
        .order_by_asc(storage_channels::Column::SortOrder)
        .order_by_asc(storage_channels::Column::Id);
    if let Some(id) = params.id {
        query = query.filter(storage_channels::Column::Id.eq(id));
    }
    if let Some(provider) = params.provider {
        query = query
            .filter(storage_channels::Column::Provider.eq(normalize_storage_provider(&provider)?));
    }
    if let Some(status) = params.status {
        validate_status(status, "storage channel")?;
        query = query.filter(storage_channels::Column::Status.eq(status));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let list = rows
        .into_iter()
        .map(StorageChannelInfo::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ApiResponse::success(PagingResponse { list, total, page }))
}

pub async fn get_storage_channel_by_id_impl(
    state: &AppState,
    id: i32,
) -> Result<StorageChannelInfo, AppError> {
    let channel = storage_channels::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("storage_channels", Some(id)))?;
    StorageChannelInfo::try_from(channel)
}

#[handler]
pub async fn get_version_manifest(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<Value>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let app = find_app(state, id.into_inner()).await?;
    Ok(ApiResponse::success(build_manifest(&app)))
}

#[handler]
pub async fn sync_app_version(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<SyncVersionReq>,
) -> Result<ApiResponse<SyncVersionResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let response = sync_app_version_impl(state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(response))
}

pub async fn sync_app_version_impl(
    state: &AppState,
    id: i32,
    req: SyncVersionReq,
) -> Result<SyncVersionResponse, AppError> {
    let app = find_app(state, id).await?;
    let manifest = build_manifest(&app);
    let channels = find_channels_for_sync(state, req.channel_ids.unwrap_or_default()).await?;
    if channels.is_empty() {
        return Err(AppError::validation("no storage channel is available"));
    }

    let mut results = Vec::with_capacity(channels.len());
    for channel in channels {
        results.push(sync_to_channel(state, &app, &manifest, channel).await?);
    }
    Ok(SyncVersionResponse {
        app_id: app.id,
        results,
    })
}

#[handler]
pub async fn list_version_sync_logs(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<VersionSyncLogInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListVersionSyncLogsParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = app_version_sync_logs::Entity::find()
        .order_by_desc(app_version_sync_logs::Column::CreatedAt)
        .order_by_desc(app_version_sync_logs::Column::Id);
    if let Some(app_id) = params.app_id {
        query = query.filter(app_version_sync_logs::Column::AppId.eq(app_id));
    }
    if let Some(storage_channel_id) = params.storage_channel_id {
        query =
            query.filter(app_version_sync_logs::Column::StorageChannelId.eq(storage_channel_id));
    }
    if let Some(provider) = params.provider {
        query = query.filter(
            app_version_sync_logs::Column::Provider.eq(normalize_storage_provider(&provider)?),
        );
    }
    if let Some(status) = params.status {
        validate_log_status(status)?;
        query = query.filter(app_version_sync_logs::Column::Status.eq(status));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    Ok(ApiResponse::success(PagingResponse {
        list: rows.into_iter().map(VersionSyncLogInfo::from).collect(),
        total,
        page,
    }))
}

async fn find_app(state: &AppState, id: i32) -> Result<apps::Model, AppError> {
    apps::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("apps", Some(id)))
}

fn build_manifest(app: &apps::Model) -> Value {
    let mut manifest = Map::new();
    manifest.insert("schema_version".to_string(), json!(1));
    manifest.insert("app_id".to_string(), json!(app.app_id));
    manifest.insert("name".to_string(), json!(app.name));
    manifest.insert(
        "version".to_string(),
        json!({
            "name": app.app_vername,
            "code": app.app_vercode,
        }),
    );
    manifest.insert("update_info".to_string(), json!(app.app_update_info));
    manifest.insert("download_url".to_string(), json!(app.app_download_url));
    if !app.app_res_url.trim().is_empty() {
        manifest.insert("res_url".to_string(), json!(app.app_res_url));
    }
    if app
        .manifest_extra
        .as_object()
        .is_some_and(|extra| !extra.is_empty())
    {
        manifest.insert("extra".to_string(), app.manifest_extra.clone());
    }
    manifest.insert(
        "published_at".to_string(),
        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    Value::Object(manifest)
}

async fn find_channels_for_sync(
    state: &AppState,
    channel_ids: Vec<i32>,
) -> Result<Vec<storage_channels::Model>, AppError> {
    let mut query = storage_channels::Entity::find()
        .order_by_asc(storage_channels::Column::SortOrder)
        .order_by_asc(storage_channels::Column::Id);
    if channel_ids.is_empty() {
        query = query
            .filter(storage_channels::Column::Status.eq(i16::from(StorageChannelStatus::Enabled)));
    } else {
        query = query
            .filter(storage_channels::Column::Id.is_in(channel_ids))
            .filter(storage_channels::Column::Status.eq(i16::from(StorageChannelStatus::Enabled)));
    }
    Ok(query.all(&state.db).await?)
}

async fn sync_to_channel(
    state: &AppState,
    app: &apps::Model,
    manifest: &Value,
    channel: storage_channels::Model,
) -> Result<SyncVersionResult, AppError> {
    let config = parse_storage_config(&channel.provider, &channel.config)?;
    let object_key = build_object_key(&config.prefix, &app.app_id);
    let public_url = build_public_url(&config.public_base_url, &object_key);
    let now = Utc::now().fixed_offset();
    let pending = app_version_sync_logs::ActiveModel {
        app_id: Set(app.id),
        storage_channel_id: Set(channel.id),
        provider: Set(channel.provider.clone()),
        object_key: Set(object_key.clone()),
        public_url: Set(public_url.clone()),
        manifest: Set(manifest.clone()),
        status: Set(LOG_STATUS_PENDING),
        error_message: Set(None),
        etag: Set(None),
        created_at: Set(now),
        finished_at: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    let body = serde_json::to_vec_pretty(manifest)
        .map_err(|error| AppError::validation(format!("failed to serialize manifest: {error}")))?;
    let upload = upload_manifest(&channel.provider, &config, &object_key, &body).await;
    let finished_at = Utc::now().fixed_offset();
    let mut active = pending.into_active_model();
    let (success, status, etag, error_message) = match upload {
        Ok(result) => (true, LOG_STATUS_SUCCESS, result.etag, None),
        Err(error) => (false, LOG_STATUS_FAILED, None, Some(error.to_string())),
    };
    active.status = Set(status);
    active.error_message = Set(error_message.clone());
    active.etag = Set(etag.clone());
    active.finished_at = Set(Some(finished_at));
    let saved = active.update(&state.db).await?;

    Ok(SyncVersionResult {
        channel_id: channel.id,
        channel_name: channel.name,
        provider: channel.provider,
        object_key,
        public_url,
        success,
        status,
        etag,
        error_message,
        log_id: saved.id,
    })
}

async fn upload_manifest(
    provider: &str,
    config: &StorageChannelConfig,
    object_key: &str,
    body: &[u8],
) -> Result<UploadResult, AppError> {
    let registry = default_registry();
    let adapter_provider = if config.endpoint.starts_with("mock://") {
        PROVIDER_MOCK
    } else {
        provider
    };
    let adapter = registry.get(adapter_provider)?;
    adapter
        .upload(UploadRequest {
            config,
            object_key,
            body,
            content_type: "application/json",
            private: false,
        })
        .await
        .map_err(AppError::from)
}

fn normalize_required_text(value: String, field: &str) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(AppError::validation(format!("{field} is required")));
    }
    Ok(value)
}

fn normalize_storage_config(provider: &str, config: &Value) -> Result<Value, AppError> {
    serde_json::to_value(parse_storage_config(provider, config)?)
        .map_err(|error| AppError::validation(format!("invalid storage config: {error}")))
}

fn response_storage_config(provider: &str, config: &Value) -> Result<Value, AppError> {
    normalize_storage_config(provider, config)
}

fn validate_status(status: i16, subject: &str) -> Result<(), AppError> {
    match status {
        0 | 1 => Ok(()),
        _ => Err(AppError::validation(format!(
            "{subject} status is not supported"
        ))),
    }
}

fn validate_log_status(status: i16) -> Result<(), AppError> {
    match status {
        LOG_STATUS_PENDING | LOG_STATUS_SUCCESS | LOG_STATUS_FAILED => Ok(()),
        _ => Err(AppError::validation(
            "version sync log status is not supported",
        )),
    }
}
