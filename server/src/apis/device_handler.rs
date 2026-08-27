use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use data_model::{app_devices, apps, reg_code_devices, reg_codes};
use salvo::prelude::*;
use salvo_oapi::ToSchema;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct DeviceInfo {
    pub id: i32,
    pub app_id: i32,
    pub app_name: String,
    pub device_id: String,
    pub device_info: Option<sea_orm::prelude::Json>,
    pub expire_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub remaining: Option<i32>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl TryFrom<(app_devices::Model, Option<apps::Model>)> for DeviceInfo {
    type Error = AppError;
    fn try_from(value: (app_devices::Model, Option<apps::Model>)) -> Result<Self, Self::Error> {
        let (app_device, app) = value;
        Ok(Self {
            id: app_device.id,
            app_id: app_device.app_id,
            app_name: app.map(|a| a.name).unwrap_or_default(),
            device_id: app_device.device_id,
            device_info: app_device.device_info,
            expire_time: app_device.expire_time,
            remaining: app_device.remaining,
            created_at: app_device.created_at,
            updated_at: app_device.updated_at,
        })
    }
}

async fn enrich_shared_remaining(
    state: &AppState,
    mut info: DeviceInfo,
) -> Result<DeviceInfo, AppError> {
    let reg_code_ids = reg_code_devices::Entity::find()
        .filter(reg_code_devices::Column::DeviceId.eq(info.id))
        .select_only()
        .column(reg_code_devices::Column::RegCodeId)
        .into_tuple::<i32>()
        .all(&state.db)
        .await?;
    if reg_code_ids.is_empty() {
        return Ok(info);
    }
    let shared_codes = reg_codes::Entity::find()
        .filter(reg_codes::Column::Id.is_in(reg_code_ids))
        .filter(reg_codes::Column::MultiDeviceEnabled.eq(true))
        .filter(reg_codes::Column::Status.eq(2))
        .filter(reg_codes::Column::CodeType.eq(1))
        .all(&state.db)
        .await?;
    let has_shared_entitlement = !shared_codes.is_empty();
    let shared_remaining: i32 = shared_codes
        .into_iter()
        .map(|reg_code| reg_code.remaining_count.unwrap_or(0).max(0))
        .sum();
    if has_shared_entitlement {
        info.remaining = Some(info.remaining.unwrap_or(0).max(0) + shared_remaining);
    }
    Ok(info)
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchDevicesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub app_id: Option<i32>,
    pub device_id: Option<String>,
}
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<DeviceInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchDevicesParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchDevicesParams,
) -> Result<PagingResponse<DeviceInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = app_devices::Entity::find()
        .find_also_related(apps::Entity)
        .order_by_desc(app_devices::Column::CreatedAt);
    if let Some(v) = params.app_id {
        query = query.filter(app_devices::Column::AppId.eq(v));
    }
    if let Some(v) = params.device_id {
        query = query.filter(app_devices::Column::DeviceId.eq(v));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let result = paginator.fetch_page(page - 1).await?;
    let mut list = Vec::with_capacity(result.len());
    for item in result {
        let info = DeviceInfo::try_from(item)?;
        list.push(enrich_shared_remaining(state, info).await?);
    }
    Ok(PagingResponse { list, total, page })
}
