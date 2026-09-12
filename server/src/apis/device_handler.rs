use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use data_model::{app_devices, apps, reg_code_devices, reg_codes};
use salvo::prelude::*;
use salvo_oapi::ToSchema;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct DeviceRegCodeInfo {
    pub id: i32,
    pub code: String,
    pub status: i16,
    pub code_type: i16,
    pub binding_time: chrono::DateTime<chrono::FixedOffset>,
    pub effective_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub expire_time: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct DeviceInfo {
    pub id: i32,
    pub app_id: i32,
    pub app_name: String,
    pub device_id: String,
    pub device_info: Option<sea_orm::prelude::Json>,
    pub expire_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub remaining: Option<i32>,
    pub bound_reg_code_count: u64,
    pub reg_codes: Vec<DeviceRegCodeInfo>,
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
            bound_reg_code_count: 0,
            reg_codes: Vec::new(),
            created_at: app_device.created_at,
            updated_at: app_device.updated_at,
        })
    }
}

async fn enrich_reg_code_remaining_batch(
    state: &AppState,
    infos: &mut [DeviceInfo],
) -> Result<(), AppError> {
    if infos.is_empty() {
        return Ok(());
    }

    let device_ids = infos.iter().map(|info| info.id).collect::<Vec<_>>();
    let bindings = reg_code_devices::Entity::find()
        .filter(reg_code_devices::Column::DeviceId.is_in(device_ids))
        .all(&state.db)
        .await?;
    if bindings.is_empty() {
        return Ok(());
    }

    let reg_code_ids = bindings
        .iter()
        .map(|binding| binding.reg_code_id)
        .collect::<HashSet<_>>();
    let reg_codes_by_id = reg_codes::Entity::find()
        .filter(reg_codes::Column::Id.is_in(reg_code_ids))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|reg_code| (reg_code.id, reg_code))
        .collect::<HashMap<_, _>>();
    let mut reg_code_remaining_by_device = HashMap::<i32, i32>::new();
    let mut reg_codes_by_device = HashMap::<i32, Vec<DeviceRegCodeInfo>>::new();
    for binding in bindings {
        let Some(reg_code) = reg_codes_by_id.get(&binding.reg_code_id) else {
            continue;
        };
        if reg_code.status == 2 && reg_code.code_type == 1 {
            *reg_code_remaining_by_device
                .entry(binding.device_id)
                .or_default() += reg_code.remaining_count.unwrap_or(0).max(0);
        }
        reg_codes_by_device
            .entry(binding.device_id)
            .or_default()
            .push(DeviceRegCodeInfo {
                id: reg_code.id,
                code: reg_code.code.clone(),
                status: reg_code.status,
                code_type: reg_code.code_type,
                binding_time: binding.created_at,
                effective_time: reg_code.effective_time,
                expire_time: reg_code.expire_time,
            });
    }

    for info in infos {
        if let Some(reg_code_remaining) = reg_code_remaining_by_device.get(&info.id) {
            info.remaining = Some(info.remaining.unwrap_or(0).max(0) + reg_code_remaining);
        }
        info.reg_codes = reg_codes_by_device.remove(&info.id).unwrap_or_default();
        info.reg_codes
            .sort_by(|a, b| b.binding_time.cmp(&a.binding_time));
        info.bound_reg_code_count = info.reg_codes.len() as u64;
    }
    Ok(())
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchDevicesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub app_id: Option<i32>,
    pub device_id: Option<String>,
    pub sort_by: Option<DeviceSortBy>,
    pub sort_order: Option<DeviceSortOrder>,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeviceSortBy {
    #[default]
    CreatedAt,
    BoundRegCodeCount,
}

#[derive(Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeviceSortOrder {
    Asc,
    #[default]
    Desc,
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
    let mut query = app_devices::Entity::find().find_also_related(apps::Entity);
    if let Some(v) = params.app_id {
        query = query.filter(app_devices::Column::AppId.eq(v));
    }
    if let Some(v) = params.device_id {
        query = query.filter(app_devices::Column::DeviceId.eq(v));
    }
    let sort_order = match params.sort_order.unwrap_or_default() {
        DeviceSortOrder::Asc => Order::Asc,
        DeviceSortOrder::Desc => Order::Desc,
    };
    query = match params.sort_by.unwrap_or_default() {
        DeviceSortBy::CreatedAt => {
            query.order_by(app_devices::Column::CreatedAt, sort_order.clone())
        }
        DeviceSortBy::BoundRegCodeCount => query.order_by(
            Expr::cust(
                r#"(SELECT COUNT(*) FROM "reg_code_devices" WHERE "reg_code_devices"."device_id" = "app_devices"."id")"#,
            ),
            sort_order.clone(),
        ),
    };
    query = query.order_by(app_devices::Column::Id, sort_order);
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let result = paginator.fetch_page(page - 1).await?;
    let mut list = result
        .into_iter()
        .map(DeviceInfo::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    enrich_reg_code_remaining_batch(state, &mut list).await?;
    Ok(PagingResponse { list, total, page })
}
