use crate::apis::list_api::*;
use crate::core::app::*;
use crate::core::my_error::*;
use crate::core::response::*;
use crate::utils::convert::from_str_optional;
use data_model::{app_version_sync_logs, apps, storage_channels};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use validator::Validate;

const LOG_STATUS_SUCCESS: i16 = 1;

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
    pub manifest_extra: Option<Value>,
    pub code_type: Option<i16>,
    pub app_valid_key: Option<String>,
    pub trial_days: Option<i32>,
    pub trial_num: Option<i32>,
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
    pub manifest_extra: Option<Value>,
    pub code_type: Option<i16>,
    pub app_valid_key: Option<String>,
    pub trial_days: Option<i32>,
    pub trial_num: Option<i32>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

#[derive(Serialize)]
pub struct AppListResponse {
    pub list: Vec<apps::Model>,
    pub total: u64,
}

#[derive(Serialize)]
pub struct AppInfo {
    #[serde(flatten)]
    pub app: apps::Model,
    pub manifest_urls: Vec<AppManifestUrlInfo>,
}

#[derive(Serialize, Clone)]
pub struct AppManifestUrlInfo {
    pub channel_id: i32,
    pub channel_name: String,
    pub provider: String,
    pub public_url: String,
    pub object_key: String,
    pub synced_at: String,
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
    let code_type = normalize_code_type(req.code_type)?;
    let (trial_days, trial_num) = normalize_trial_limits(code_type, req.trial_days, req.trial_num);
    let manifest_extra = normalize_manifest_extra(req.manifest_extra)?;
    let active_model = apps::ActiveModel {
        name: Set(req.name),
        app_id: Set(req.app_id),
        app_vername: Set(req.app_vername),
        app_vercode: Set(req.app_vercode),
        app_download_url: Set(req.app_download_url),
        app_res_url: Set(req.app_res_url),
        app_update_info: Set(req.app_update_info),
        manifest_extra: Set(manifest_extra),
        code_type: Set(code_type),
        app_valid_key: Set(req.app_valid_key.unwrap_or_default()),
        trial_days: Set(trial_days),
        trial_num: Set(trial_num),
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
    let final_code_type = normalize_code_type(req.code_type.or(Some(app.code_type)))?;
    let (final_trial_days, final_trial_num) = normalize_trial_limits(
        final_code_type,
        req.trial_days.or(Some(app.trial_days)),
        req.trial_num.or(Some(app.trial_num)),
    );
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
    if let Some(v) = req.manifest_extra {
        app.manifest_extra = Set(normalize_manifest_extra(Some(v))?);
    }
    if req.code_type.is_some() {
        app.code_type = Set(final_code_type);
    }
    if let Some(v) = req.app_valid_key {
        app.app_valid_key = Set(v);
    }
    app.trial_days = Set(final_trial_days);
    app.trial_num = Set(final_trial_num);
    if let Some(v) = req.sort_order {
        app.sort_order = Set(v);
    }
    if let Some(v) = req.status {
        app.status = Set(v);
    }
    let app = app.update(&state.db).await?;
    Ok(app)
}

fn normalize_code_type(value: Option<i16>) -> Result<i16, AppError> {
    match value.unwrap_or_default() {
        0 => Ok(0),
        1 => Ok(1),
        _ => Err(AppError::validation("code_type must be 0 or 1")),
    }
}

fn normalize_trial_limits(
    code_type: i16,
    trial_days: Option<i32>,
    trial_num: Option<i32>,
) -> (i32, i32) {
    match code_type {
        1 => (0, trial_num.unwrap_or_default()),
        _ => (trial_days.unwrap_or_default(), 0),
    }
}

fn normalize_manifest_extra(value: Option<Value>) -> Result<Value, AppError> {
    let value = value.unwrap_or_else(|| Value::Object(Map::new()));
    match value {
        Value::Null => Ok(Value::Object(Map::new())),
        Value::Object(map) => {
            let mut normalized = Map::new();
            for (key, value) in map {
                let key = key.trim();
                if key.is_empty() {
                    continue;
                }
                normalized.insert(key.to_string(), value);
            }
            Ok(Value::Object(normalized))
        }
        _ => Err(AppError::validation("manifest_extra must be a JSON object")),
    }
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
) -> Result<ApiResponse<PagingResponse<AppInfo>>, AppError> {
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
) -> Result<PagingResponse<AppInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;
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
    let manifest_urls_by_app =
        manifest_urls_for_apps(state, list.iter().map(|app| app.id).collect::<Vec<_>>()).await?;
    let list = list
        .into_iter()
        .map(|app| AppInfo {
            manifest_urls: manifest_urls_by_app
                .get(&app.id)
                .cloned()
                .unwrap_or_default(),
            app,
        })
        .collect();
    Ok(PagingResponse { list, total, page })
}

async fn manifest_urls_for_apps(
    state: &AppState,
    app_ids: Vec<i32>,
) -> Result<HashMap<i32, Vec<AppManifestUrlInfo>>, AppError> {
    if app_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = app_version_sync_logs::Entity::find()
        .filter(app_version_sync_logs::Column::AppId.is_in(app_ids))
        .filter(app_version_sync_logs::Column::Status.eq(LOG_STATUS_SUCCESS))
        .order_by_desc(app_version_sync_logs::Column::CreatedAt)
        .order_by_desc(app_version_sync_logs::Column::Id)
        .find_also_related(storage_channels::Entity)
        .all(&state.db)
        .await?;

    let mut latest_by_channel = HashMap::new();
    for (log, channel) in rows {
        latest_by_channel
            .entry((log.app_id, log.storage_channel_id))
            .or_insert_with(|| {
                let channel_name = channel
                    .as_ref()
                    .map(|channel| channel.name.clone())
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| log.provider.clone());
                AppManifestUrlInfo {
                    channel_id: log.storage_channel_id,
                    channel_name,
                    provider: log.provider,
                    public_url: log.public_url,
                    object_key: log.object_key,
                    synced_at: log.created_at.to_rfc3339(),
                }
            });
    }

    let mut urls_by_app: HashMap<i32, Vec<AppManifestUrlInfo>> = HashMap::new();
    for ((app_id, _channel_id), info) in latest_by_channel {
        urls_by_app.entry(app_id).or_default().push(info);
    }
    for urls in urls_by_app.values_mut() {
        urls.sort_by(|a, b| a.channel_id.cmp(&b.channel_id));
    }
    Ok(urls_by_app)
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
