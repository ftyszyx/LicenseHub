use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use data_model::{apps, use_records};
use salvo::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug)]
pub struct UseRecordPublicInfo {
    pub id: i32,
    pub app_id: i32,
    pub device_id: String,
    pub use_count: i32,
    pub use_info: Option<sea_orm::prelude::Json>,
    pub time: i64,
}

impl From<use_records::Model> for UseRecordPublicInfo {
    fn from(value: use_records::Model) -> Self {
        Self {
            id: value.id,
            app_id: value.app_id,
            device_id: value.device_id,
            use_count: value.use_count,
            use_info: value.use_info,
            time: value.time.timestamp(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UseRecordAdminInfo {
    pub id: i32,
    pub app_id: i32,
    pub app_name: Option<String>,
    pub device_id: String,
    pub use_count: i32,
    pub use_info: Option<sea_orm::prelude::Json>,
    pub time: i64,
}

impl From<(use_records::Model, Option<apps::Model>)> for UseRecordAdminInfo {
    fn from(value: (use_records::Model, Option<apps::Model>)) -> Self {
        let (record, app) = value;
        Self {
            id: record.id,
            app_id: record.app_id,
            app_name: app.map(|item| item.name),
            device_id: record.device_id,
            use_count: record.use_count,
            use_info: record.use_info,
            time: record.time.timestamp(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct PublicUseRecordsReq {
    pub app_key: String,
    pub device_id: String,
    #[serde(flatten)]
    pub pagination: ListParamsReq,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchUseRecordsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub app_id: Option<i32>,
    pub device_id: Option<String>,
}

pub async fn create_use_record<C>(
    db: &C,
    app_id: i32,
    device_id: &str,
    use_count: i32,
    use_info: Option<sea_orm::prelude::Json>,
) -> Result<use_records::Model, AppError>
where
    C: ConnectionTrait,
{
    let active_model = use_records::ActiveModel {
        app_id: Set(app_id),
        device_id: Set(device_id.to_string()),
        use_count: Set(use_count),
        use_info: Set(use_info),
        ..Default::default()
    };
    active_model.insert(db).await.map_err(AppError::from)
}

#[handler]
pub async fn public_get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<UseRecordPublicInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<PublicUseRecordsReq>()?;
    let list = public_get_list_impl(state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn public_get_list_impl(
    state: &AppState,
    params: PublicUseRecordsReq,
) -> Result<PagingResponse<UseRecordPublicInfo>, AppError> {
    let app_model = apps::Entity::find()
        .filter(apps::Column::AppValidKey.eq(params.app_key))
        .one(&state.db)
        .await?
        .ok_or(AppError::not_found("apps".to_string(), None))?;

    let (page, page_size) = params.pagination.resolve()?;
    let query = use_records::Entity::find()
        .filter(
            use_records::Column::AppId
                .eq(app_model.id)
                .and(use_records::Column::DeviceId.eq(params.device_id)),
        )
        .order_by_desc(use_records::Column::Time);

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let list = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(UseRecordPublicInfo::from)
        .collect();

    Ok(PagingResponse { list, total, page })
}

#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<UseRecordAdminInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchUseRecordsParams>()?;
    let list = get_list_impl(state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchUseRecordsParams,
) -> Result<PagingResponse<UseRecordAdminInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;

    let mut query = use_records::Entity::find()
        .find_also_related(apps::Entity)
        .order_by_desc(use_records::Column::Time);

    if let Some(app_id) = params.app_id {
        query = query.filter(use_records::Column::AppId.eq(app_id));
    }
    if let Some(device_id) = params.device_id {
        query = query.filter(use_records::Column::DeviceId.contains(device_id));
    }

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let list = paginator
        .fetch_page(page - 1)
        .await?
        .into_iter()
        .map(UseRecordAdminInfo::from)
        .collect();

    Ok(PagingResponse { list, total, page })
}
