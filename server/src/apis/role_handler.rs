use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::Utc;
use data_model::roles;
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct RoleCreatePayload {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct RoleUpdatePayload {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct RoleListResponse {
    pub list: Vec<roles::Model>,
    pub total: u64,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListRolesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub id: Option<i32>,
    pub name: Option<String>,
}

// Create Role
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<RoleCreatePayload>,
) -> Result<ApiResponse<roles::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: RoleCreatePayload) -> Result<roles::Model, AppError> {
    let now = Utc::now().fixed_offset();
    let active_model = roles::ActiveModel {
        name: Set(req.name),
        description: Set(req.description),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let entity = active_model.insert(&state.db).await?;
    Ok(entity)
}

// Update Role
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<RoleUpdatePayload>,
) -> Result<ApiResponse<roles::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let role = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(role))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: RoleUpdatePayload,
) -> Result<roles::Model, AppError> {
    let role = roles::Entity::find_by_id(id).one(&state.db).await?;
    let role = role.ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;
    let mut role: roles::ActiveModel = role.into_active_model();
    if let Some(v) = req.name {
        role.name = Set(v);
    }
    if let Some(v) = req.description {
        role.description = Set(Some(v));
    }
    role.updated_at = Set(Utc::now().fixed_offset());
    let role = role.update(&state.db).await?;
    Ok(role)
}

// Delete Role
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    delete_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let role = roles::Entity::find_by_id(id).one(&state.db).await?;
    let role = role.ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;
    let _ = role.into_active_model().delete(&state.db).await?;
    Ok(())
}

// Get Roles List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<roles::Model>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListRolesParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: ListRolesParams,
) -> Result<PagingResponse<roles::Model>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = roles::Entity::find().order_by_desc(roles::Column::CreatedAt);
    if let Some(v) = params.name {
        query = query.filter(roles::Column::Name.contains(v));
    }
    if let Some(v) = params.id {
        query = query.filter(roles::Column::Id.eq(v));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let list = paginator.fetch_page(page - 1).await?;
    Ok(PagingResponse { list, total, page })
}

// Get Role by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<roles::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let role = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(role))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<roles::Model, AppError> {
    let query = roles::Entity::find_by_id(id).one(&state.db).await?;
    let role = query.ok_or_else(|| AppError::not_found("roles".to_string(), Some(id)))?;
    Ok(role)
}
