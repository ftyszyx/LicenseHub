use crate::apis::list_api::*;
use crate::core::app::*;
use crate::core::my_error::*;
use crate::core::response::*;
use data_model::apps;
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

fn get_state(depot: &mut Depot) -> Result<&AppState, AppError> {
    depot
        .obtain::<AppState>()
        .map_err(|_| AppError::InternalError {
            message: "missing AppState in depot".to_string(),
        })
}

#[derive(Serialize, Deserialize, Debug, Default, Validate)]
pub struct AddAppReq {
    pub name: String,
    pub app_id: String,
    pub app_vername: String,
    pub app_vercode: i32,
    pub app_download_url: String,
    pub app_res_url: String,
    pub app_update_info: Option<String>,
    pub app_valid_key: Option<String>,
    pub trial_days: Option<i32>,
    pub sort_order: i32,
    pub status: i16,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UpdateAppReq {
    pub name: Option<String>,
    pub app_id: Option<String>,
    pub app_vername: Option<String>,
    pub app_vercode: Option<i32>,
    pub app_download_url: Option<String>,
    pub app_res_url: Option<String>,
    pub app_update_info: Option<String>,
    pub app_valid_key: Option<String>,
    pub trial_days: Option<i32>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

#[derive(Serialize)]
pub struct AppListResponse {
    pub list: Vec<apps::Model>,
    pub total: u64,
}

#[derive(Deserialize, Debug, Default)]
pub struct ListAppsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub id: Option<i32>,
    pub app_id: Option<String>,
    pub name: Option<String>,
}

// Create App
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<AddAppReq>,
) -> Result<ApiResponse<apps::Model>, AppError> {
    let state = get_state(depot)?;
    let req = req.into_inner();
    req.validate()?;
    let entity = add_impl(state, req).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: AddAppReq) -> Result<apps::Model, AppError> {
    let active_model = apps::ActiveModel {
        name: Set(req.name),
        app_id: Set(req.app_id),
        app_vername: Set(req.app_vername),
        app_vercode: Set(req.app_vercode),
        app_download_url: Set(req.app_download_url),
        app_res_url: Set(req.app_res_url),
        app_update_info: Set(req.app_update_info),
        app_valid_key: Set(req.app_valid_key.unwrap_or_default()),
        trial_days: Set(req.trial_days.unwrap_or_default()),
        sort_order: Set(req.sort_order),
        status: Set(req.status),
        ..Default::default()
    };
    let entity = active_model.insert(&state.db).await?;
    Ok(entity)
}

#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    json: JsonBody<UpdateAppReq>,
) -> Result<ApiResponse<apps::Model>, AppError> {
    let state = get_state(depot)?;
    let req = json.into_inner();
    req.validate()?;
    let app = update_impl(state, id.into_inner(), req).await?;
    Ok(ApiResponse::success(app))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: UpdateAppReq,
) -> Result<apps::Model, AppError> {
    let app = apps::Entity::find_by_id(id).one(&state.db).await?;
    let app = app.ok_or_else(|| AppError::not_found("apps".to_string(), Some(id)))?;
    let mut app: apps::ActiveModel = app.into_active_model();
    if let Some(v) = req.name {
        app.name = Set(v);
    }
    if let Some(v) = req.app_id {
        app.app_id = Set(v);
    }
    if let Some(v) = req.app_vername {
        app.app_vername = Set(v);
    }
    if let Some(v) = req.app_vercode {
        app.app_vercode = Set(v);
    }
    if let Some(v) = req.app_download_url {
        app.app_download_url = Set(v);
    }
    if let Some(v) = req.app_res_url {
        app.app_res_url = Set(v);
    }
    if let Some(v) = req.app_update_info {
        app.app_update_info = Set(Some(v));
    }
    if let Some(v) = req.app_valid_key {
        app.app_valid_key = Set(v);
    }
    if let Some(v) = req.trial_days {
        app.trial_days = Set(v);
    }
    if let Some(v) = req.sort_order {
        app.sort_order = Set(v);
    }
    if let Some(v) = req.status {
        app.status = Set(v);
    }
    let app = app.update(&state.db).await?;
    Ok(app)
}

#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = get_state(depot)?;
    let id = id.into_inner();
    delete_impl(state, id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let app = apps::Entity::find_by_id(id).one(&state.db).await?;
    let app = app.ok_or_else(|| AppError::not_found("apps".to_string(), Some(id)))?;
    let _ = app.into_active_model().delete(&state.db).await?;
    Ok(())
}

// Get Apps List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<apps::Model>>, AppError> {
    let state = get_state(depot)?;
    let params = req
        .parse_queries::<ListAppsParams>()
        .map_err(AppError::from)?;
    let list = get_list_impl(state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: ListAppsParams,
) -> Result<PagingResponse<apps::Model>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);
    let mut query = apps::Entity::find().order_by_desc(apps::Column::CreatedAt);

    if let Some(name) = params.name {
        query = query.filter(apps::Column::Name.contains(name));
    }
    if let Some(id) = params.id {
        query = query.filter(apps::Column::Id.eq(id));
    }
    if let Some(app_id) = params.app_id {
        query = query.filter(apps::Column::AppId.contains(app_id));
    }

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let list = paginator.fetch_page(page - 1).await?;
    Ok(PagingResponse { list, total, page })
}

// Get App by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<apps::Model>, AppError> {
    let state = get_state(depot)?;
    let id = id.into_inner();
    let app = get_by_id_impl(state, id).await?;
    Ok(ApiResponse::success(app))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<apps::Model, AppError> {
    let query = apps::Entity::find_by_id(id).one(&state.db).await?;
    let app = query.ok_or_else(|| AppError::not_found("apps".to_string(), Some(id)))?;
    Ok(app)
}
