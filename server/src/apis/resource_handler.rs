use crate::apis::auth_middleware::Claims;
use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::resource_service::{
    MAX_RESOURCE_SIZE, ResourceUpload, delete_resource, download_resource, normalize_resource_type,
    upload_resource,
};
use crate::core::response::ApiResponse;
use data_model::{order_refund_attachments, resources, storage_channels};
use salvo::oapi::extract::PathParam;
use salvo::prelude::*;
use sea_orm::{
    ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct ResourceInfo {
    pub id: i64,
    pub storage_channel_id: i32,
    pub storage_channel_name: Option<String>,
    pub resource_type: String,
    pub original_name: String,
    pub content_type: String,
    pub size: i64,
    pub uploaded_by: i32,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<resources::Model> for ResourceInfo {
    fn from(value: resources::Model) -> Self {
        Self::from_model(value, None)
    }
}

impl ResourceInfo {
    fn from_model(value: resources::Model, storage_channel_name: Option<String>) -> Self {
        Self {
            id: value.id,
            storage_channel_id: value.storage_channel_id,
            storage_channel_name,
            resource_type: value.resource_type,
            original_name: value.original_name,
            content_type: value.content_type,
            size: value.size,
            uploaded_by: value.uploaded_by,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ListResourcesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub resource_type: Option<String>,
    pub keyword: Option<String>,
}

#[handler]
pub async fn list_resources(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<ResourceInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListResourcesParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = resources::Entity::find()
        .find_also_related(storage_channels::Entity)
        .order_by_desc(resources::Column::CreatedAt)
        .order_by_desc(resources::Column::Id);
    if let Some(resource_type) = params.resource_type {
        let resource_type = resource_type.trim();
        if !resource_type.is_empty() {
            query = query.filter(
                resources::Column::ResourceType.eq(normalize_resource_type(resource_type)?),
            );
        }
    }
    if let Some(keyword) = params.keyword {
        let keyword = keyword.trim();
        if !keyword.is_empty() {
            query = query.filter(
                resources::Column::OriginalName
                    .contains(keyword)
                    .or(resources::Column::ObjectKey.contains(keyword)),
            );
        }
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    Ok(ApiResponse::success(PagingResponse {
        list: rows
            .into_iter()
            .map(|(resource, channel)| {
                ResourceInfo::from_model(resource, channel.map(|value| value.name))
            })
            .collect(),
        total,
        page,
    }))
}

#[handler]
pub async fn upload_resource_handler(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<ResourceInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let form = req.form_data().await?;
    let resource_type = form
        .fields
        .get("resource_type")
        .cloned()
        .unwrap_or_default();
    let file = form
        .files
        .get("file")
        .ok_or_else(|| AppError::validation("file is required"))?;
    if file.size() > MAX_RESOURCE_SIZE {
        return Err(AppError::validation("resource must not exceed 20 MB"));
    }
    let content = tokio::fs::read(file.path())
        .await
        .map_err(|error| AppError::InternalError {
            message: format!("failed to read resource: {error}"),
        })?;
    let content_type = file
        .content_type()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let tx = state.db.begin().await?;
    let resource = upload_resource(
        state,
        &tx,
        claims.user_id,
        ResourceUpload {
            resource_type,
            original_name: file.name().unwrap_or("resource").to_string(),
            content_type,
            content,
        },
    )
    .await?;
    tx.commit().await?;
    let storage_channel_name = storage_channels::Entity::find_by_id(resource.storage_channel_id)
        .one(&state.db)
        .await?
        .map(|channel| channel.name);
    Ok(ApiResponse::success(ResourceInfo::from_model(
        resource,
        storage_channel_name,
    )))
}

#[handler]
pub async fn download_resource_handler(
    depot: &mut Depot,
    id: PathParam<i64>,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let resource = resources::Entity::find_by_id(id.into_inner())
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("resources", None))?;
    let content_type = resource.content_type.clone();
    let downloaded = download_resource(state, &resource).await?;
    res.headers_mut().insert(
        salvo::http::header::CONTENT_TYPE,
        salvo::http::HeaderValue::from_str(&content_type)
            .map_err(|_| AppError::validation("invalid resource content type"))?,
    );
    res.headers_mut().insert(
        salvo::http::header::CONTENT_DISPOSITION,
        salvo::http::HeaderValue::from_static("inline"),
    );
    res.headers_mut().insert(
        salvo::http::header::CACHE_CONTROL,
        salvo::http::HeaderValue::from_static("private, no-store"),
    );
    res.write_body(downloaded.body)
        .map_err(|error| AppError::InternalError {
            message: format!("failed to write resource: {error}"),
        })?;
    Ok(())
}

#[handler]
pub async fn delete_resource_handler(
    depot: &mut Depot,
    id: PathParam<i64>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let id = id.into_inner();
    let resource = resources::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("resources", None))?;
    let referenced = order_refund_attachments::Entity::find()
        .filter(order_refund_attachments::Column::ResourceId.eq(id))
        .one(&state.db)
        .await?;
    if referenced.is_some() {
        return Err(AppError::business_logic(
            "RESOURCE_IN_USE",
            "resource is referenced by a refund record and cannot be deleted",
        ));
    }
    delete_resource(state, resource).await?;
    Ok(ApiResponse::success(()))
}
