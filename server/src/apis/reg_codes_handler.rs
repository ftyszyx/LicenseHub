use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::apis::system_settings_handler::get_license_signing_private_key_b64;
use crate::apis::use_record_handler::create_use_record;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::license_signing::{
    LicensePayload, SignedLicense, app_key_hash, public_key_b64_from_private_key, sign_license,
};
use chrono::{DateTime, FixedOffset, Utc};
use data_model::{app_devices, apps, reg_code_devices, reg_codes};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use salvo_oapi::{ToSchema, endpoint};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum CodeType {
    Time = 0,  // 时间类型
    Count = 1, // 次数类型
}

impl Default for CodeType {
    fn default() -> Self {
        CodeType::Time
    }
}

impl From<i16> for CodeType {
    fn from(value: i16) -> Self {
        match value {
            0 => CodeType::Time,
            1 => CodeType::Count,
            _ => CodeType::Time,
        }
    }
}

impl From<CodeType> for i16 {
    fn from(value: CodeType) -> Self {
        value as i16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum RegCodeStatus {
    Unused = 0,
    Issued = 1,
    Binded = 2,
    Refunded = 3,
    Revoked = 4,
}

impl Default for RegCodeStatus {
    fn default() -> Self {
        RegCodeStatus::Unused
    }
}

impl From<i16> for RegCodeStatus {
    fn from(value: i16) -> Self {
        match value {
            0 => RegCodeStatus::Unused,
            1 => RegCodeStatus::Issued,
            2 => RegCodeStatus::Binded,
            3 => RegCodeStatus::Refunded,
            4 => RegCodeStatus::Revoked,
            _ => RegCodeStatus::Unused,
        }
    }
}

impl From<RegCodeStatus> for i16 {
    fn from(value: RegCodeStatus) -> Self {
        value as i16
    }
}

#[derive(Serialize, Deserialize, Debug, Validate, Default, ToSchema)]
pub struct CreateRegCodeReq {
    pub code: String,
    pub app_id: i32,
    pub valid_days: i32,
    pub max_devices: Option<i32>,
    pub status: RegCodeStatus,
    pub code_type: CodeType,
    pub expire_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub total_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct RegCodeValidateReq {
    pub code: Option<String>,
    pub app_key: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RegCodeValidateResp {
    pub code_type: CodeType,
    pub expire_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub remaining_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct RegCodeBindReq {
    pub app_key: String,
    #[serde(alias = "code")]
    pub reg_code: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct RegCodeCheckReq {
    pub app_key: String,
    pub device_id: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RegCodeStatusResp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expire_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remain_count: Option<i32>,
    pub license: SignedLicense,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct UseCountReq {
    pub app_key: String,
    pub device_id: String,
    pub use_count: i32,
    pub use_info: Option<sea_orm::prelude::Json>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UseCountResp {
    pub remain_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct UpdateRegCodeReq {
    pub code: Option<String>,
    pub app_id: Option<i32>,
    pub valid_days: Option<i32>,
    pub max_devices: Option<i32>,
    pub status: Option<i16>,
    pub code_type: Option<CodeType>,
    pub total_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct UpdateRegCodeStatusReq {
    pub status: RegCodeStatus,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchRegCodesParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    #[serde(default)]
    pub id: Option<i32>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub app_id: Option<i32>,
    #[serde(default)]
    pub status: Option<i16>,
    #[serde(default)]
    pub code_type: Option<CodeType>,
    #[serde(default)]
    pub device_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Validate, ToSchema)]
pub struct RegCodeInfo {
    pub id: i32,
    pub code: String,
    pub app_id: i32,
    pub valid_days: i32,
    pub max_devices: i32,
    pub status: i16,
    pub binding_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub code_type: CodeType,
    pub total_count: Option<i32>,
    pub remaining_count: Option<i32>,
    pub device_id: Option<i32>,
    pub device_id_str: Option<String>,
    pub device_ids: Vec<String>,
    pub bound_device_count: u64,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub app_name: Option<String>,
    pub device_info: Option<sea_orm::prelude::Json>,
}

impl
    TryFrom<(
        reg_codes::Model,
        Option<apps::Model>,
        Option<app_devices::Model>,
    )> for RegCodeInfo
{
    type Error = AppError;

    fn try_from(
        value: (
            reg_codes::Model,
            Option<apps::Model>,
            Option<app_devices::Model>,
        ),
    ) -> Result<Self, Self::Error> {
        let (reg_code, app, device) = value;
        let device_id_str = device.as_ref().map(|d| d.device_id.clone());
        let device_info = device.as_ref().and_then(|d| d.device_info.clone());
        Ok(Self {
            id: reg_code.id,
            code: reg_code.code,
            app_id: reg_code.app_id,
            valid_days: reg_code.valid_days,
            max_devices: reg_code.max_devices,
            status: reg_code.status,
            binding_time: reg_code.binding_time,
            code_type: CodeType::from(reg_code.code_type),
            total_count: reg_code.total_count,
            remaining_count: reg_code.remaining_count,
            device_id: reg_code.device_id,
            device_id_str,
            device_ids: device
                .as_ref()
                .map(|value| vec![value.device_id.clone()])
                .unwrap_or_default(),
            bound_device_count: u64::from(device.is_some()),
            device_info,
            created_at: reg_code.created_at,
            updated_at: reg_code.updated_at,
            app_name: app.map(|a| a.name),
        })
    }
}

impl TryFrom<reg_codes::Model> for RegCodeInfo {
    type Error = AppError;

    fn try_from(reg_code: reg_codes::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: reg_code.id,
            code: reg_code.code,
            app_id: reg_code.app_id,
            valid_days: reg_code.valid_days,
            max_devices: reg_code.max_devices,
            status: reg_code.status,
            binding_time: reg_code.binding_time,
            code_type: CodeType::from(reg_code.code_type),
            total_count: reg_code.total_count,
            remaining_count: reg_code.remaining_count,
            device_id: reg_code.device_id,
            device_id_str: None,
            device_ids: Vec::new(),
            bound_device_count: 0,
            device_info: None,
            created_at: reg_code.created_at,
            updated_at: reg_code.updated_at,
            app_name: None,
        })
    }
}

async fn enrich_reg_code_devices(
    state: &AppState,
    info: RegCodeInfo,
) -> Result<RegCodeInfo, AppError> {
    let reg_code_id = info.id;
    let mut devices_by_reg_code = load_reg_code_device_ids(&state.db, &[reg_code_id]).await?;
    Ok(with_reg_code_devices(
        info,
        devices_by_reg_code.remove(&reg_code_id).unwrap_or_default(),
    ))
}

async fn load_reg_code_device_ids<C>(
    db: &C,
    reg_code_ids: &[i32],
) -> Result<HashMap<i32, Vec<String>>, AppError>
where
    C: ConnectionTrait,
{
    if reg_code_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = reg_code_devices::Entity::find()
        .filter(reg_code_devices::Column::RegCodeId.is_in(reg_code_ids.to_vec()))
        .find_also_related(app_devices::Entity)
        .all(db)
        .await?;

    let mut devices_by_reg_code = HashMap::<i32, Vec<String>>::new();
    for (binding, device) in rows {
        if let Some(device) = device {
            devices_by_reg_code
                .entry(binding.reg_code_id)
                .or_default()
                .push(device.device_id);
        }
    }
    for device_ids in devices_by_reg_code.values_mut() {
        device_ids.sort();
        device_ids.dedup();
    }
    Ok(devices_by_reg_code)
}

fn with_reg_code_devices(mut info: RegCodeInfo, mut device_ids: Vec<String>) -> RegCodeInfo {
    device_ids.sort();
    device_ids.dedup();
    if device_ids.is_empty() {
        if let Some(device_id) = info.device_id_str.clone() {
            device_ids.push(device_id);
        }
    }
    info.bound_device_count = device_ids.len() as u64;
    info.device_id_str = device_ids.first().cloned();
    info.device_ids = device_ids;
    info
}

// Create RegCode
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<RegCodeInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let json = req.parse_json::<CreateRegCodeReq>().await?;
    let entity = add_impl(&state, json).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: CreateRegCodeReq) -> Result<RegCodeInfo, AppError> {
    let (code_type, valid_days, total_count, app_max_devices) = normalize_reg_code_limits_for_app(
        state,
        req.app_id,
        req.code_type,
        req.valid_days,
        req.total_count,
    )
    .await?;
    let max_devices = req.max_devices.unwrap_or(app_max_devices);
    if max_devices < 1 {
        return Err(AppError::validation(
            "max_devices must be greater than or equal to 1",
        ));
    }
    let now = Utc::now().fixed_offset();
    let active_model = reg_codes::ActiveModel {
        code: Set(req.code),
        app_id: Set(req.app_id),
        valid_days: Set(valid_days),
        max_devices: Set(max_devices),
        status: Set(i16::from(req.status)),
        code_type: Set(i16::from(code_type)),
        total_count: Set(total_count),
        remaining_count: Set(total_count),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let entity = active_model.insert(&state.db).await?;

    // Fetch with app information for response
    let result = reg_codes::Entity::find_by_id(entity.id)
        .find_also_related(apps::Entity)
        .find_also_related(app_devices::Entity)
        .one(&state.db)
        .await?;

    match result {
        Some((reg_code, app, device)) => {
            enrich_reg_code_devices(state, RegCodeInfo::try_from((reg_code, app, device))?).await
        }
        None => Err(AppError::not_found(
            "reg_codes".to_string(),
            Some(entity.id),
        )),
    }
}

// Update RegCode
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UpdateRegCodeReq>,
) -> Result<ApiResponse<RegCodeInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let reg_code = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(reg_code))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: UpdateRegCodeReq,
) -> Result<RegCodeInfo, AppError> {
    let tx = state.db.begin().await?;
    let reg_code = reg_codes::Entity::find_by_id(id)
        .lock_exclusive()
        .one(&tx)
        .await?;
    let reg_code =
        reg_code.ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;
    if matches!(
        RegCodeStatus::from(reg_code.status),
        RegCodeStatus::Refunded | RegCodeStatus::Revoked
    ) {
        return Err(AppError::business_logic(
            "REG_CODE_STATUS_LOCKED",
            "refunded or revoked reg code cannot be changed",
        ));
    }
    let final_app_id = req.app_id.unwrap_or(reg_code.app_id);
    let final_code_type = req
        .code_type
        .unwrap_or_else(|| CodeType::from(reg_code.code_type));
    let final_valid_days = req.valid_days.unwrap_or(reg_code.valid_days);
    let final_total_count = req.total_count.or(reg_code.total_count);
    if let Some(max_devices) = req.max_devices {
        if max_devices < 1 {
            return Err(AppError::validation(
                "max_devices must be greater than or equal to 1",
            ));
        }

        let mut bound_device_ids = reg_code_devices::Entity::find()
            .filter(reg_code_devices::Column::RegCodeId.eq(reg_code.id))
            .select_only()
            .column(reg_code_devices::Column::DeviceId)
            .into_tuple::<i32>()
            .all(&tx)
            .await?;
        if let Some(device_id) = reg_code.device_id {
            bound_device_ids.push(device_id);
        }
        bound_device_ids.sort_unstable();
        bound_device_ids.dedup();

        if max_devices < bound_device_ids.len() as i32 {
            return Err(AppError::validation(
                "max_devices cannot be lower than the current bound device count",
            ));
        }

        let has_binding = !bound_device_ids.is_empty()
            || RegCodeStatus::from(reg_code.status) == RegCodeStatus::Binded;
        if (max_devices > 1) != (reg_code.max_devices > 1) && has_binding {
            return Err(AppError::validation(
                "cannot switch single-device and multi-device mode after reg code is bound",
            ));
        }
    }
    let refresh_reg_code_remaining = final_code_type == CodeType::Count
        && RegCodeStatus::from(reg_code.status) != RegCodeStatus::Binded;
    let (code_type, valid_days, total_count, _) = normalize_reg_code_limits_for_app(
        state,
        final_app_id,
        final_code_type,
        final_valid_days,
        final_total_count,
    )
    .await?;

    let mut reg_code: reg_codes::ActiveModel = reg_code.into_active_model();
    if let Some(v) = req.code {
        reg_code.code = Set(v);
    }
    reg_code.app_id = Set(final_app_id);
    reg_code.valid_days = Set(valid_days);
    if let Some(v) = req.max_devices {
        reg_code.max_devices = Set(v);
    }
    if let Some(v) = req.status {
        let status = RegCodeStatus::from(v);
        if matches!(
            status,
            RegCodeStatus::Binded | RegCodeStatus::Refunded | RegCodeStatus::Revoked
        ) {
            return Err(AppError::validation(
                "cannot set reg_code status to binded, refunded, or revoked directly",
            ));
        }
        reg_code.status = Set(v);
    }
    reg_code.code_type = Set(i16::from(code_type));
    reg_code.total_count = Set(total_count);
    if refresh_reg_code_remaining {
        reg_code.remaining_count = Set(total_count);
    } else if final_code_type == CodeType::Time {
        reg_code.remaining_count = Set(None);
    }
    reg_code.updated_at = Set(Utc::now().fixed_offset());

    let updated_reg_code = reg_code.update(&tx).await?;
    tx.commit().await?;

    // Fetch with app information for response
    let result = reg_codes::Entity::find_by_id(updated_reg_code.id)
        .find_also_related(apps::Entity)
        .find_also_related(app_devices::Entity)
        .one(&state.db)
        .await?;

    match result {
        Some((reg_code, app, device)) => {
            enrich_reg_code_devices(state, RegCodeInfo::try_from((reg_code, app, device))?).await
        }
        None => Err(AppError::not_found(
            "reg_codes".to_string(),
            Some(updated_reg_code.id),
        )),
    }
}

async fn normalize_reg_code_limits_for_app(
    state: &AppState,
    app_id: i32,
    code_type: CodeType,
    valid_days: i32,
    total_count: Option<i32>,
) -> Result<(CodeType, i32, Option<i32>, i32), AppError> {
    let app = apps::Entity::find_by_id(app_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("apps".to_string(), Some(app_id)))?;
    if CodeType::from(app.code_type) != code_type {
        return Err(AppError::validation(
            "reg_code code_type must match app code_type",
        ));
    }
    match code_type {
        CodeType::Time => {
            if valid_days <= 0 {
                return Err(AppError::validation("valid_days must be greater than 0"));
            }
            Ok((code_type, valid_days, None, app.max_devices))
        }
        CodeType::Count => {
            let total_count = total_count.unwrap_or(0);
            if total_count <= 0 {
                return Err(AppError::validation("total_count must be greater than 0"));
            }
            Ok((code_type, 0, Some(total_count), app.max_devices))
        }
    }
}

// Update RegCode Status
#[handler]
pub async fn update_status(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UpdateRegCodeStatusReq>,
) -> Result<ApiResponse<RegCodeInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let reg_code = update_status_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(reg_code))
}

pub async fn update_status_impl(
    state: &AppState,
    id: i32,
    req: UpdateRegCodeStatusReq,
) -> Result<RegCodeInfo, AppError> {
    let reg_code = reg_codes::Entity::find_by_id(id).one(&state.db).await?;
    let reg_code =
        reg_code.ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;

    if matches!(
        RegCodeStatus::from(reg_code.status),
        RegCodeStatus::Binded | RegCodeStatus::Refunded | RegCodeStatus::Revoked
    ) {
        return Err(AppError::business_logic(
            "REG_CODE_STATUS_LOCKED",
            "binded, refunded, or revoked reg code status cannot be changed",
        ));
    }

    if req.status == RegCodeStatus::Binded {
        return Err(AppError::validation("cannot set reg_code status to binded"));
    }
    if matches!(req.status, RegCodeStatus::Refunded | RegCodeStatus::Revoked) {
        return Err(AppError::validation(
            "cannot set reg_code status to refunded or revoked directly",
        ));
    }

    let mut active: reg_codes::ActiveModel = reg_code.into_active_model();
    active.status = Set(i16::from(req.status));
    active.updated_at = Set(Utc::now().fixed_offset());
    let updated_reg_code = active.update(&state.db).await?;

    let result = reg_codes::Entity::find_by_id(updated_reg_code.id)
        .find_also_related(apps::Entity)
        .find_also_related(app_devices::Entity)
        .one(&state.db)
        .await?;

    match result {
        Some((reg_code, app, device)) => {
            enrich_reg_code_devices(state, RegCodeInfo::try_from((reg_code, app, device))?).await
        }
        None => Err(AppError::not_found(
            "reg_codes".to_string(),
            Some(updated_reg_code.id),
        )),
    }
}

#[handler]
pub async fn revoke(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<RegCodeInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let reg_code = revoke_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(reg_code))
}

pub async fn revoke_impl(state: &AppState, id: i32) -> Result<RegCodeInfo, AppError> {
    let tx = state.db.begin().await?;
    let reg_code = reg_codes::Entity::find_by_id(id)
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;

    match RegCodeStatus::from(reg_code.status) {
        RegCodeStatus::Unused | RegCodeStatus::Issued | RegCodeStatus::Binded => {}
        RegCodeStatus::Refunded => {
            return Err(AppError::business_logic(
                "REG_CODE_ALREADY_REFUNDED",
                "reg code is already refunded",
            ));
        }
        RegCodeStatus::Revoked => {
            return Err(AppError::business_logic(
                "REG_CODE_ALREADY_REVOKED",
                "reg code authorization is already revoked",
            ));
        }
    }

    let updated_reg_code = apply_reg_code_revocation(&tx, reg_code, RegCodeStatus::Revoked).await?;
    tx.commit().await?;

    get_by_id_impl(state, updated_reg_code.id).await
}

pub(crate) async fn revoke_reg_code_for_order(
    tx: &DatabaseTransaction,
    id: i32,
) -> Result<(), AppError> {
    let reg_code = reg_codes::Entity::find_by_id(id)
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;

    if RegCodeStatus::from(reg_code.status) == RegCodeStatus::Refunded {
        return Ok(());
    }
    apply_reg_code_revocation(tx, reg_code, RegCodeStatus::Refunded).await?;
    Ok(())
}

async fn apply_reg_code_revocation(
    tx: &DatabaseTransaction,
    reg_code: reg_codes::Model,
    target_status: RegCodeStatus,
) -> Result<reg_codes::Model, AppError> {
    let now = Utc::now().fixed_offset();
    let mut device_ids = reg_code_devices::Entity::find()
        .filter(reg_code_devices::Column::RegCodeId.eq(reg_code.id))
        .select_only()
        .column(reg_code_devices::Column::DeviceId)
        .into_tuple::<i32>()
        .all(tx)
        .await?;
    if let Some(legacy_device_id) = reg_code.device_id {
        device_ids.push(legacy_device_id);
    }
    device_ids.sort_unstable();
    device_ids.dedup();

    if !device_ids.is_empty() {
        let devices = app_devices::Entity::find()
            .filter(app_devices::Column::Id.is_in(device_ids))
            .order_by_asc(app_devices::Column::Id)
            .lock_exclusive()
            .all(tx)
            .await?;
        for device in devices {
            let mut active_device = device.into_active_model();
            active_device.expire_time = Set(Some(now));
            active_device.remaining = Set(Some(0));
            active_device.updated_at = Set(now);
            active_device.update(tx).await?;
        }
    }

    let clear_reg_code_remaining = CodeType::from(reg_code.code_type) == CodeType::Count;
    let mut active_reg_code = reg_code.into_active_model();
    active_reg_code.status = Set(i16::from(target_status));
    active_reg_code.device_id = Set(None);
    if clear_reg_code_remaining {
        active_reg_code.remaining_count = Set(Some(0));
    }
    active_reg_code.updated_at = Set(now);
    Ok(active_reg_code.update(tx).await?)
}

// Delete RegCode
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    delete_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let reg_code = reg_codes::Entity::find_by_id(id).one(&state.db).await?;
    let reg_code =
        reg_code.ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;
    if RegCodeStatus::from(reg_code.status) != RegCodeStatus::Unused {
        return Err(AppError::business_logic(
            "REG_CODE_DELETE_FORBIDDEN",
            "only unused reg codes can be deleted",
        ));
    }
    reg_code.into_active_model().delete(&state.db).await?;
    Ok(())
}

// Get RegCodes List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<RegCodeInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchRegCodesParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchRegCodesParams,
) -> Result<PagingResponse<RegCodeInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;

    let mut query = reg_codes::Entity::find()
        .find_also_related(apps::Entity)
        .find_also_related(app_devices::Entity)
        .order_by_desc(reg_codes::Column::CreatedAt);

    if let Some(v) = params.id {
        query = query.filter(reg_codes::Column::Id.eq(v));
    }
    if let Some(v) = params.code {
        query = query.filter(reg_codes::Column::Code.contains(v));
    }
    if let Some(v) = params.app_id {
        query = query.filter(reg_codes::Column::AppId.eq(v));
    }
    if let Some(v) = params.status {
        query = query.filter(reg_codes::Column::Status.eq(v));
    }
    if let Some(v) = params.code_type {
        query = query.filter(reg_codes::Column::CodeType.eq(i16::from(v)));
    }
    if let Some(v) = params.device_id {
        let device_ids = app_devices::Entity::find()
            .filter(app_devices::Column::DeviceId.contains(v))
            .select_only()
            .column(app_devices::Column::Id)
            .into_tuple::<i32>()
            .all(&state.db)
            .await?;
        let reg_code_ids = if device_ids.is_empty() {
            Vec::new()
        } else {
            reg_code_devices::Entity::find()
                .filter(reg_code_devices::Column::DeviceId.is_in(device_ids.clone()))
                .select_only()
                .column(reg_code_devices::Column::RegCodeId)
                .into_tuple::<i32>()
                .all(&state.db)
                .await?
        };
        query = query.filter(
            Condition::any()
                .add(reg_codes::Column::Id.is_in(reg_code_ids))
                .add(reg_codes::Column::DeviceId.is_in(device_ids)),
        );
    }

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let results = paginator.fetch_page(page - 1).await?;

    let reg_code_ids = results
        .iter()
        .map(|(reg_code, _, _)| reg_code.id)
        .collect::<Vec<_>>();
    let mut devices_by_reg_code = load_reg_code_device_ids(&state.db, &reg_code_ids).await?;
    let mut list = Vec::with_capacity(results.len());
    for (reg_code, app, device) in results {
        let reg_code_id = reg_code.id;
        let info = RegCodeInfo::try_from((reg_code, app, device))?;
        list.push(with_reg_code_devices(
            info,
            devices_by_reg_code.remove(&reg_code_id).unwrap_or_default(),
        ));
    }

    Ok(PagingResponse { list, total, page })
}

// Get RegCode by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<RegCodeInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let reg_code = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(reg_code))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<RegCodeInfo, AppError> {
    let result = reg_codes::Entity::find_by_id(id)
        .find_also_related(apps::Entity)
        .find_also_related(app_devices::Entity)
        .one(&state.db)
        .await?;

    match result {
        Some((reg_code, app, device)) => {
            enrich_reg_code_devices(state, RegCodeInfo::try_from((reg_code, app, device))?).await
        }
        None => Err(AppError::not_found("reg_codes".to_string(), Some(id))),
    }
}

/// Validate registration code for device
// #[handler]
// refer https://github.com/salvo-rs/salvo/blob/main/crates/oapi/docs/endpoint.md
#[endpoint(tags("reg_codes"))]
pub async fn validate_code(
    depot: &mut Depot,
    req: JsonBody<RegCodeValidateReq>,
) -> Result<ApiResponse<RegCodeValidateResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let resp = validate_code_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(resp))
}

/// Validate registration code for device (GET)
// #[handler]
#[endpoint(
    tags( "reg_codes" ),
    parameters(
        ("code"=Option<String>,Query, description = "注册码"),
    ("app_key"=String, Query, description = "应用校验Key"),
    ("device_id"=String, Query, description = "设备ID")
))]
pub async fn validate_code_get(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<RegCodeValidateResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let json = req.parse_queries::<RegCodeValidateReq>()?;
    let resp = validate_code_impl(&state, json).await?;
    Ok(ApiResponse::success(resp))
}

#[endpoint(tags("reg_codes"))]
pub async fn bind_code(
    depot: &mut Depot,
    req: JsonBody<RegCodeBindReq>,
) -> Result<ApiResponse<RegCodeStatusResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let resp = bind_code_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(resp))
}

#[endpoint(tags("reg_codes"))]
pub async fn bind_code_get(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<RegCodeStatusResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let json = req.parse_queries::<RegCodeBindReq>()?;
    let resp = bind_code_impl(&state, json).await?;
    Ok(ApiResponse::success(resp))
}

#[endpoint(tags("reg_codes"))]
pub async fn check_device(
    depot: &mut Depot,
    req: JsonBody<RegCodeCheckReq>,
) -> Result<ApiResponse<RegCodeStatusResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let resp = check_device_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(resp))
}

#[endpoint(tags("reg_codes"))]
pub async fn check_device_get(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<RegCodeStatusResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let json = req.parse_queries::<RegCodeCheckReq>()?;
    let resp = check_device_impl(&state, json).await?;
    Ok(ApiResponse::success(resp))
}

#[handler]
pub async fn use_count(
    depot: &mut Depot,
    req: JsonBody<UseCountReq>,
) -> Result<ApiResponse<UseCountResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let resp = use_count_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(resp))
}

async fn find_app_by_key(state: &AppState, app_key: &str) -> Result<apps::Model, AppError> {
    apps::Entity::find()
        .filter(apps::Column::AppValidKey.eq(app_key.to_string()))
        .one(&state.db)
        .await?
        .ok_or(AppError::not_found("apps".to_string(), None))
}

async fn find_device_by_app_and_id(
    state: &AppState,
    app_id: i32,
    device_id: &str,
) -> Result<Option<app_devices::Model>, AppError> {
    app_devices::Entity::find()
        .filter(
            app_devices::Column::AppId
                .eq(app_id)
                .and(app_devices::Column::DeviceId.eq(device_id.to_string())),
        )
        .one(&state.db)
        .await
        .map_err(AppError::from)
}

async fn find_reg_code_by_app_and_code(
    state: &AppState,
    app_id: i32,
    code: &str,
) -> Result<reg_codes::Model, AppError> {
    reg_codes::Entity::find()
        .filter(
            reg_codes::Column::AppId
                .eq(app_id)
                .and(reg_codes::Column::Code.eq(code.to_string())),
        )
        .one(&state.db)
        .await?
        .ok_or(AppError::not_found("reg_code".to_string(), None))
}

fn time_status_resp(
    state: &AppState,
    private_key_b64: &str,
    app_model: &apps::Model,
    device_id: &str,
    expire_time: DateTime<FixedOffset>,
) -> Result<RegCodeStatusResp, AppError> {
    let expire_time_seconds = expire_time.timestamp();
    Ok(RegCodeStatusResp {
        expire_time: Some(expire_time_seconds),
        remain_count: None,
        license: status_license(
            state,
            private_key_b64,
            app_model,
            device_id,
            "time",
            Some(expire_time_seconds),
            None,
        )?,
    })
}

fn count_status_resp(
    state: &AppState,
    private_key_b64: &str,
    app_model: &apps::Model,
    device_id: &str,
    remain_count: i32,
) -> Result<RegCodeStatusResp, AppError> {
    Ok(RegCodeStatusResp {
        expire_time: None,
        remain_count: Some(remain_count),
        license: status_license(
            state,
            private_key_b64,
            app_model,
            device_id,
            "count",
            None,
            Some(remain_count),
        )?,
    })
}

fn status_license(
    state: &AppState,
    private_key_b64: &str,
    app_model: &apps::Model,
    device_id: &str,
    license_type: &str,
    expire_time: Option<i64>,
    remain_count: Option<i32>,
) -> Result<SignedLicense, AppError> {
    sign_license(
        &state.config.license_signing,
        private_key_b64,
        LicensePayload {
            version: 1,
            app_id: app_model.app_id.clone(),
            app_key_hash: app_key_hash(&app_model.app_valid_key),
            device_id: device_id.to_string(),
            license_type: license_type.to_string(),
            expire_time,
            remain_count,
            issued_at: Utc::now().timestamp(),
        },
    )
}

async fn require_license_signing_private_key_b64(state: &AppState) -> Result<String, AppError> {
    let private_key_b64 = get_license_signing_private_key_b64(state)
        .await?
        .ok_or_else(|| AppError::Message("license signing key is not configured".to_string()))?;
    public_key_b64_from_private_key(&private_key_b64)?;
    Ok(private_key_b64)
}

fn ensure_reg_code_bound_to_current_device(
    device_model: Option<&app_devices::Model>,
    reg_code_model: &reg_codes::Model,
) -> Result<(), AppError> {
    match (reg_code_model.device_id, device_model) {
        (Some(bound_device_id), Some(device_model)) if device_model.id == bound_device_id => Ok(()),
        (Some(_), _) => Err(AppError::Message(
            "reg code already bound to another device".into(),
        )),
        (None, _) => Ok(()),
    }
}

enum BoundEntitlement {
    Time(DateTime<FixedOffset>),
    Count(i32),
}

async fn find_device_by_app_and_id_on<C>(
    db: &C,
    app_id: i32,
    device_id: &str,
) -> Result<Option<app_devices::Model>, AppError>
where
    C: ConnectionTrait,
{
    Ok(app_devices::Entity::find()
        .filter(
            app_devices::Column::AppId
                .eq(app_id)
                .and(app_devices::Column::DeviceId.eq(device_id.to_string())),
        )
        .one(db)
        .await?)
}

async fn code_balanced_count_reg_codes_for_device<C>(
    db: &C,
    device_id: i32,
) -> Result<Vec<reg_codes::Model>, AppError>
where
    C: ConnectionTrait,
{
    let reg_code_ids = reg_code_devices::Entity::find()
        .filter(reg_code_devices::Column::DeviceId.eq(device_id))
        .select_only()
        .column(reg_code_devices::Column::RegCodeId)
        .into_tuple::<i32>()
        .all(db)
        .await?;
    if reg_code_ids.is_empty() {
        return Ok(Vec::new());
    }

    Ok(reg_codes::Entity::find()
        .filter(reg_codes::Column::Id.is_in(reg_code_ids))
        .filter(reg_codes::Column::Status.eq(i16::from(RegCodeStatus::Binded)))
        .filter(reg_codes::Column::CodeType.eq(i16::from(CodeType::Count)))
        .order_by_asc(reg_codes::Column::Id)
        .all(db)
        .await?)
}

async fn effective_device_remaining<C>(db: &C, device: &app_devices::Model) -> Result<i32, AppError>
where
    C: ConnectionTrait,
{
    let reg_code_remaining: i32 = code_balanced_count_reg_codes_for_device(db, device.id)
        .await?
        .into_iter()
        .map(|reg_code| reg_code.remaining_count.unwrap_or(0).max(0))
        .sum();
    Ok(device.remaining.unwrap_or(0).max(0) + reg_code_remaining)
}

async fn bind_code_level_entitlement(
    state: &AppState,
    app_model: &apps::Model,
    device_id: &str,
    reg_code_id: i32,
    consume_count: i32,
) -> Result<BoundEntitlement, AppError> {
    if consume_count < 0 {
        return Err(AppError::validation(
            "consume_count must be greater than or equal to 0",
        ));
    }

    let tx = state.db.begin().await?;
    let reg_code = reg_codes::Entity::find_by_id(reg_code_id)
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("reg_code", Some(reg_code_id)))?;

    match RegCodeStatus::from(reg_code.status) {
        RegCodeStatus::Refunded => {
            return Err(AppError::business_logic(
                "REG_CODE_REFUNDED",
                "reg code has been refunded",
            ));
        }
        RegCodeStatus::Revoked => {
            return Err(AppError::business_logic(
                "REG_CODE_REVOKED",
                "reg code authorization has been revoked",
            ));
        }
        _ => {}
    }

    let now = Utc::now().fixed_offset();
    let mut device = app_devices::Entity::find()
        .filter(
            app_devices::Column::AppId
                .eq(app_model.id)
                .and(app_devices::Column::DeviceId.eq(device_id.to_string())),
        )
        .lock_exclusive()
        .one(&tx)
        .await?;
    let already_bound = match device.as_ref() {
        Some(device) => reg_code_devices::Entity::find_by_id((reg_code.id, device.id))
            .one(&tx)
            .await?
            .is_some(),
        None => false,
    };

    if !already_bound {
        let bound_count = reg_code_devices::Entity::find()
            .filter(reg_code_devices::Column::RegCodeId.eq(reg_code.id))
            .count(&tx)
            .await?;
        if bound_count >= reg_code.max_devices as u64 {
            return Err(AppError::business_logic(
                "REG_CODE_DEVICE_LIMIT_REACHED",
                "reg code device limit reached",
            ));
        }
    }

    let entitlement = match CodeType::from(reg_code.code_type) {
        CodeType::Time => {
            if consume_count != 0 {
                return Err(AppError::validation(
                    "consume_count is only supported for count reg codes",
                ));
            }
            if already_bound {
                let expire_time = device
                    .as_ref()
                    .and_then(|value| value.expire_time)
                    .unwrap_or(now);
                if expire_time <= now {
                    return Err(AppError::Message("device expired".into()));
                }
                BoundEntitlement::Time(expire_time)
            } else {
                let expire_time = match device.take() {
                    Some(device_model) => {
                        let current_expire = device_model.expire_time.unwrap_or(now).max(now);
                        let expire_time =
                            current_expire + chrono::Duration::days(reg_code.valid_days as i64);
                        let mut active_device = device_model.into_active_model();
                        active_device.expire_time = Set(Some(expire_time));
                        active_device.updated_at = Set(now);
                        device = Some(active_device.update(&tx).await?);
                        expire_time
                    }
                    None => {
                        let expire_time = now + chrono::Duration::days(reg_code.valid_days as i64);
                        device = Some(
                            app_devices::ActiveModel {
                                app_id: Set(app_model.id),
                                device_id: Set(device_id.to_string()),
                                expire_time: Set(Some(expire_time)),
                                ..Default::default()
                            }
                            .insert(&tx)
                            .await?,
                        );
                        expire_time
                    }
                };
                BoundEntitlement::Time(expire_time)
            }
        }
        CodeType::Count => {
            let remaining = reg_code.remaining_count.unwrap_or(0);
            if remaining <= 0 || remaining < consume_count {
                return Err(AppError::Message(
                    "reg code remaining count is not enough".into(),
                ));
            }
            if device.is_none() {
                device = Some(
                    app_devices::ActiveModel {
                        app_id: Set(app_model.id),
                        device_id: Set(device_id.to_string()),
                        ..Default::default()
                    }
                    .insert(&tx)
                    .await?,
                );
            }
            BoundEntitlement::Count(remaining - consume_count)
        }
    };

    let device = device.ok_or_else(|| AppError::Message("device not found".into()))?;
    if !already_bound {
        reg_code_devices::ActiveModel {
            reg_code_id: Set(reg_code.id),
            device_id: Set(device.id),
            created_at: Set(now),
        }
        .insert(&tx)
        .await?;
    }

    let had_binding_time = reg_code.binding_time.is_some();
    let had_legacy_device = reg_code.device_id.is_some();
    let mut active_reg_code = reg_code.into_active_model();
    if !had_binding_time {
        active_reg_code.binding_time = Set(Some(now));
    }
    active_reg_code.status = Set(i16::from(RegCodeStatus::Binded));
    if !had_legacy_device {
        active_reg_code.device_id = Set(Some(device.id));
    }
    if let BoundEntitlement::Count(remaining) = &entitlement {
        active_reg_code.remaining_count = Set(Some(*remaining));
    }
    active_reg_code.updated_at = Set(now);
    active_reg_code.update(&tx).await?;

    let entitlement = match entitlement {
        BoundEntitlement::Time(expire_time) => BoundEntitlement::Time(expire_time),
        BoundEntitlement::Count(_) => {
            BoundEntitlement::Count(effective_device_remaining(&tx, &device).await?)
        }
    };
    tx.commit().await?;
    Ok(entitlement)
}

pub async fn bind_code_impl(
    state: &AppState,
    req: RegCodeBindReq,
) -> Result<RegCodeStatusResp, AppError> {
    let app_model = find_app_by_key(state, &req.app_key).await?;
    let device_model = find_device_by_app_and_id(state, app_model.id, &req.device_id).await?;
    let reg_code_model = find_reg_code_by_app_and_code(state, app_model.id, &req.reg_code).await?;
    if matches!(
        RegCodeStatus::from(reg_code_model.status),
        RegCodeStatus::Refunded | RegCodeStatus::Revoked
    ) {
        return Err(AppError::business_logic(
            if RegCodeStatus::from(reg_code_model.status) == RegCodeStatus::Revoked {
                "REG_CODE_REVOKED"
            } else {
                "REG_CODE_REFUNDED"
            },
            if RegCodeStatus::from(reg_code_model.status) == RegCodeStatus::Revoked {
                "reg code authorization has been revoked"
            } else {
                "reg code has been refunded"
            },
        ));
    }
    let private_key_b64 = require_license_signing_private_key_b64(state).await?;

    let uses_code_level_entitlement = reg_code_model.max_devices > 1
        || CodeType::from(reg_code_model.code_type) == CodeType::Count;
    if uses_code_level_entitlement {
        return match bind_code_level_entitlement(
            state,
            &app_model,
            &req.device_id,
            reg_code_model.id,
            0,
        )
        .await?
        {
            BoundEntitlement::Time(expire_time) => time_status_resp(
                state,
                &private_key_b64,
                &app_model,
                &req.device_id,
                expire_time,
            ),
            BoundEntitlement::Count(remaining) => count_status_resp(
                state,
                &private_key_b64,
                &app_model,
                &req.device_id,
                remaining,
            ),
        };
    }

    bind_time_reg_code(
        state,
        &private_key_b64,
        &app_model,
        &req.device_id,
        device_model,
        &reg_code_model,
    )
    .await
}

pub async fn check_device_impl(
    state: &AppState,
    req: RegCodeCheckReq,
) -> Result<RegCodeStatusResp, AppError> {
    let app_model = find_app_by_key(state, &req.app_key).await?;
    let device_model = find_device_by_app_and_id(state, app_model.id, &req.device_id).await?;
    let private_key_b64 = require_license_signing_private_key_b64(state).await?;
    let now = Utc::now().fixed_offset();

    match CodeType::from(app_model.code_type) {
        CodeType::Time => {
            let device_model = match device_model {
                Some(device_model) => device_model,
                None => {
                    if app_model.trial_days <= 0 {
                        return Err(AppError::Message("app has no trial".into()));
                    }
                    let expire_time = now + chrono::Duration::days(app_model.trial_days as i64);
                    app_devices::ActiveModel {
                        app_id: Set(app_model.id),
                        device_id: Set(req.device_id.clone()),
                        expire_time: Set(Some(expire_time)),
                        ..Default::default()
                    }
                    .insert(&state.db)
                    .await?
                }
            };
            let expire_time = device_model.expire_time.unwrap_or(now);
            if expire_time <= now {
                return Err(AppError::Message("device expired".into()));
            }
            time_status_resp(
                state,
                &private_key_b64,
                &app_model,
                &req.device_id,
                expire_time,
            )
        }
        CodeType::Count => {
            let device_model = match device_model {
                Some(device_model) => device_model,
                None => {
                    if app_model.trial_num <= 0 {
                        return Err(AppError::Message("app has no trial".into()));
                    }
                    app_devices::ActiveModel {
                        app_id: Set(app_model.id),
                        device_id: Set(req.device_id.clone()),
                        remaining: Set(Some(app_model.trial_num)),
                        ..Default::default()
                    }
                    .insert(&state.db)
                    .await?
                }
            };
            let remain_count = effective_device_remaining(&state.db, &device_model).await?;
            if remain_count <= 0 {
                return Err(AppError::Message("device remaining count is 0".into()));
            }
            count_status_resp(
                state,
                &private_key_b64,
                &app_model,
                &req.device_id,
                remain_count,
            )
        }
    }
}

pub async fn use_count_impl(state: &AppState, req: UseCountReq) -> Result<UseCountResp, AppError> {
    if req.use_count <= 0 {
        return Err(AppError::Message("use_count must be greater than 0".into()));
    }

    let app_model = find_app_by_key(state, &req.app_key).await?;
    if CodeType::from(app_model.code_type) != CodeType::Count {
        return Err(AppError::Message("app code type is not count".into()));
    }

    let tx = state.db.begin().await?;
    let initial_device = find_device_by_app_and_id_on(&tx, app_model.id, &req.device_id).await?;
    let code_balance_ids = match initial_device.as_ref() {
        Some(device) => {
            reg_code_devices::Entity::find()
                .filter(reg_code_devices::Column::DeviceId.eq(device.id))
                .select_only()
                .column(reg_code_devices::Column::RegCodeId)
                .into_tuple::<i32>()
                .all(&tx)
                .await?
        }
        None => Vec::new(),
    };
    let mut code_balances = if code_balance_ids.is_empty() {
        Vec::new()
    } else {
        reg_codes::Entity::find()
            .filter(reg_codes::Column::Id.is_in(code_balance_ids))
            .filter(reg_codes::Column::Status.eq(i16::from(RegCodeStatus::Binded)))
            .filter(reg_codes::Column::CodeType.eq(i16::from(CodeType::Count)))
            .order_by_asc(reg_codes::Column::Id)
            .lock_exclusive()
            .all(&tx)
            .await?
    };
    let device_model = app_devices::Entity::find()
        .filter(
            app_devices::Column::AppId
                .eq(app_model.id)
                .and(app_devices::Column::DeviceId.eq(req.device_id.clone())),
        )
        .lock_exclusive()
        .one(&tx)
        .await?;

    let reg_code_before: i32 = code_balances
        .iter()
        .map(|reg_code| reg_code.remaining_count.unwrap_or(0).max(0))
        .sum();
    let device_before = device_model
        .as_ref()
        .and_then(|device| device.remaining)
        .unwrap_or(0)
        .max(0);
    let remain_before = reg_code_before + device_before;

    if device_model.is_none() && code_balances.is_empty() {
        if app_model.trial_num <= 0 {
            return Err(AppError::Message("app has no trial".into()));
        }
        if app_model.trial_num < req.use_count {
            return Err(AppError::Message(format!(
                "device remaining count is not enough: remain={}, required={}",
                app_model.trial_num, req.use_count
            )));
        }
        let remain_after = app_model.trial_num - req.use_count;
        app_devices::ActiveModel {
            app_id: Set(app_model.id),
            device_id: Set(req.device_id.clone()),
            remaining: Set(Some(remain_after)),
            ..Default::default()
        }
        .insert(&tx)
        .await?;

        create_use_record(
            &tx,
            app_model.id,
            &req.device_id,
            req.use_count,
            req.use_info,
        )
        .await?;
        tx.commit().await?;
        return Ok(UseCountResp {
            remain_count: remain_after,
        });
    }

    if remain_before < req.use_count {
        return Err(AppError::Message(format!(
            "device remaining count is not enough: remain={}, required={}",
            remain_before, req.use_count
        )));
    }

    let now = Utc::now().fixed_offset();
    let mut required = req.use_count;
    for reg_code in code_balances.drain(..) {
        if required == 0 {
            break;
        }
        let remaining = reg_code.remaining_count.unwrap_or(0).max(0);
        let consumed = remaining.min(required);
        if consumed == 0 {
            continue;
        }
        required -= consumed;
        let mut active_reg_code = reg_code.into_active_model();
        active_reg_code.remaining_count = Set(Some(remaining - consumed));
        active_reg_code.updated_at = Set(now);
        active_reg_code.update(&tx).await?;
    }

    if required > 0 {
        let device_model = device_model
            .ok_or_else(|| AppError::Message("device remaining count is not enough".into()))?;
        let mut active_device = device_model.into_active_model();
        active_device.remaining = Set(Some(device_before - required));
        active_device.updated_at = Set(now);
        active_device.update(&tx).await?;
    }
    let remain_after = remain_before - req.use_count;

    create_use_record(
        &tx,
        app_model.id,
        &req.device_id,
        req.use_count,
        req.use_info,
    )
    .await?;

    tx.commit().await?;
    Ok(UseCountResp {
        remain_count: remain_after,
    })
}

pub async fn bind_time_reg_code(
    state: &AppState,
    private_key_b64: &str,
    app_model: &apps::Model,
    device_id: &str,
    device_model: Option<app_devices::Model>,
    reg_code_model: &reg_codes::Model,
) -> Result<RegCodeStatusResp, AppError> {
    if CodeType::from(reg_code_model.code_type) != CodeType::Time {
        return Err(AppError::Message("code type is not time".into()));
    }

    let now = Utc::now().fixed_offset();
    if reg_code_model.device_id.is_some() {
        ensure_reg_code_bound_to_current_device(device_model.as_ref(), reg_code_model)?;
        let device_model =
            device_model.ok_or_else(|| AppError::Message("device not found".into()))?;
        let expire_time = device_model.expire_time.unwrap_or(now);
        if expire_time <= now {
            return Err(AppError::Message("device expired".into()));
        }
        return time_status_resp(state, private_key_b64, app_model, device_id, expire_time);
    }

    let tx = state.db.begin().await?;
    let mut active_reg_model = reg_code_model.clone().into_active_model();
    active_reg_model.binding_time = Set(Some(now));
    active_reg_model.status = Set(RegCodeStatus::Binded.into());

    let expire_time = match device_model {
        Some(device_model) => {
            let current_expire = device_model.expire_time.unwrap_or(now);
            let current_expire = if current_expire < now {
                now
            } else {
                current_expire
            };
            let new_expire_time =
                current_expire + chrono::Duration::days(reg_code_model.valid_days as i64);

            active_reg_model.device_id = Set(Some(device_model.id));
            let mut active_device_model = device_model.into_active_model();
            active_device_model.expire_time = Set(Some(new_expire_time));
            active_device_model.update(&tx).await?;
            new_expire_time
        }
        None => {
            let new_expire_time = now + chrono::Duration::days(reg_code_model.valid_days as i64);
            let device_active = app_devices::ActiveModel {
                app_id: Set(app_model.id),
                device_id: Set(device_id.to_string()),
                expire_time: Set(Some(new_expire_time)),
                ..Default::default()
            };
            let inserted_device = device_active.insert(&tx).await?;
            active_reg_model.device_id = Set(Some(inserted_device.id));
            new_expire_time
        }
    };

    active_reg_model.update(&tx).await?;
    tx.commit().await?;
    time_status_resp(state, private_key_b64, app_model, device_id, expire_time)
}

pub async fn validate_code_impl(
    state: &AppState,
    req: RegCodeValidateReq,
) -> Result<RegCodeValidateResp, AppError> {
    // find app by app_valid_key
    let app_model = apps::Entity::find()
        .filter(apps::Column::AppValidKey.eq(req.app_key.clone()))
        .one(&state.db)
        .await?
        .ok_or(AppError::not_found("apps".to_string(), None))?;

    let device_id = req.device_id.clone();
    let code_is_none = req.code.as_deref().map(|c| c.is_empty()).unwrap_or(true);
    // find or create device
    let dev = app_devices::Entity::find()
        .filter(
            app_devices::Column::AppId
                .eq(app_model.id)
                .and(app_devices::Column::DeviceId.eq(device_id.clone())),
        )
        .one(&state.db)
        .await?;
    // trial-only mode
    if code_is_none {
        return trial_validate(state, &app_model, &device_id, dev).await;
    }

    // find reg code
    let code = req.code.clone().unwrap();
    let reg_code_model = reg_codes::Entity::find()
        .filter(
            reg_codes::Column::AppId
                .eq(app_model.id)
                .and(reg_codes::Column::Code.eq(code)),
        )
        .one(&state.db)
        .await?
        .ok_or(AppError::not_found("reg_code".to_string(), None))?;
    if matches!(
        RegCodeStatus::from(reg_code_model.status),
        RegCodeStatus::Refunded | RegCodeStatus::Revoked
    ) {
        return Err(AppError::business_logic(
            if RegCodeStatus::from(reg_code_model.status) == RegCodeStatus::Revoked {
                "REG_CODE_REVOKED"
            } else {
                "REG_CODE_REFUNDED"
            },
            if RegCodeStatus::from(reg_code_model.status) == RegCodeStatus::Revoked {
                "reg code authorization has been revoked"
            } else {
                "reg code has been refunded"
            },
        ));
    }
    let uses_code_level_entitlement = reg_code_model.max_devices > 1
        || CodeType::from(reg_code_model.code_type) == CodeType::Count;
    if uses_code_level_entitlement {
        let consume_count = if CodeType::from(app_model.code_type) == CodeType::Count {
            1
        } else {
            0
        };
        return match bind_code_level_entitlement(
            state,
            &app_model,
            &device_id,
            reg_code_model.id,
            consume_count,
        )
        .await?
        {
            BoundEntitlement::Time(expire_time) => Ok(RegCodeValidateResp {
                code_type: CodeType::Time,
                expire_time: Some(expire_time),
                remaining_count: None,
            }),
            BoundEntitlement::Count(remaining_count) => Ok(RegCodeValidateResp {
                code_type: CodeType::Count,
                expire_time: None,
                remaining_count: Some(remaining_count),
            }),
        };
    }
    time_reg_code_validate(state, &app_model, &device_id, dev, &reg_code_model).await
}
//试用
pub async fn trial_validate(
    state: &AppState,
    app_model: &apps::Model,
    device_id: &str,
    device_model: Option<app_devices::Model>,
) -> Result<RegCodeValidateResp, AppError> {
    let now = Utc::now().fixed_offset();
    match app_model.code_type.into() {
        CodeType::Time => {
            let trial_expire = now + chrono::Duration::days(app_model.trial_days as i64);
            match device_model {
                Some(d) => {
                    let device_expire = d.expire_time.unwrap_or(trial_expire);
                    if now > device_expire {
                        return Err(AppError::Message("device expired".into()));
                    }
                }
                None => {
                    if app_model.trial_days <= 0 {
                        return Err(AppError::Message("app has no trial".into()));
                    }
                    // 创建新的设备记录
                    app_devices::ActiveModel {
                        app_id: Set(app_model.id),
                        device_id: Set(device_id.to_string()),
                        device_info: Set(None),
                        expire_time: Set(Some(trial_expire)),
                        ..Default::default()
                    }
                    .insert(&state.db)
                    .await?;
                }
            }
            Ok(RegCodeValidateResp {
                code_type: CodeType::Time,
                expire_time: Some(trial_expire),
                remaining_count: None,
            })
        }
        CodeType::Count => {
            let mut new_remaining = 0;
            match device_model {
                Some(d) => {
                    let remaining = d.remaining.unwrap_or(0);
                    if remaining <= 0 {
                        return Err(AppError::Message("trial used up".into()));
                    }
                    new_remaining = remaining - 1;
                }
                None => {
                    if app_model.trial_num <= 0 {
                        return Err(AppError::Message("app has no trial".into()));
                    }
                }
            }
            app_devices::ActiveModel {
                app_id: Set(app_model.id),
                device_id: Set(device_id.to_string()),
                device_info: Set(None),
                remaining: Set(Some(new_remaining)),
                ..Default::default()
            }
            .insert(&state.db)
            .await?;
            Ok(RegCodeValidateResp {
                code_type: CodeType::Count,
                expire_time: None,
                remaining_count: Some(new_remaining),
            })
        }
    }
}

//时间类型注册码验证
pub async fn time_reg_code_validate(
    state: &AppState,
    app_model: &apps::Model,
    device_id: &str,
    device_model: Option<app_devices::Model>,
    reg_code_model: &reg_codes::Model,
) -> Result<RegCodeValidateResp, AppError> {
    let now = Utc::now().fixed_offset();
    if CodeType::from(reg_code_model.code_type) != CodeType::Time {
        return Err(AppError::Message("code type is not time".into()));
    }
    // update reg_code binding if needed
    if reg_code_model.device_id.is_none() {
        let tx = state.db.begin().await?;
        let mut active_reg_model = reg_code_model.clone().into_active_model();
        active_reg_model.binding_time = Set(Some(now));
        active_reg_model.status = Set(RegCodeStatus::Binded.into());
        let expire_time: DateTime<FixedOffset> = match device_model {
            Some(dm) => {
                active_reg_model.device_id = Set(Some(dm.id));
                let device_expire = dm.expire_time.unwrap_or(now);
                //check device expire time, if it is less than now, use now as device expire time
                let device_expire = if device_expire < now {
                    now
                } else {
                    device_expire
                };
                let expire_time =
                    device_expire + chrono::Duration::days(reg_code_model.valid_days as i64);
                let mut active_device_model = dm.into_active_model();
                active_device_model.expire_time = Set(Some(expire_time));
                active_device_model.update(&tx).await?;
                expire_time
            }
            None => {
                let expire_time = now + chrono::Duration::days(reg_code_model.valid_days as i64);
                //add device
                let device_active = app_devices::ActiveModel {
                    app_id: Set(app_model.id),
                    device_id: Set(device_id.to_string()),
                    expire_time: Set(Some(expire_time)),
                    ..Default::default()
                };
                let inserted_device = device_active.insert(&tx).await?;
                active_reg_model.device_id = Set(Some(inserted_device.id));
                expire_time
            }
        };
        active_reg_model.update(&tx).await?;
        tx.commit().await?;
        return Ok(RegCodeValidateResp {
            code_type: CodeType::Time,
            expire_time: Some(expire_time),
            remaining_count: None,
        });
    }

    // update device final expire_time (cache) if needed
    let device_model = device_model.ok_or_else(|| AppError::Message("device not found".into()))?;
    let current_device_expire = device_model.expire_time.unwrap_or(now);
    if current_device_expire <= now {
        return Err(AppError::Message("device expired".into()));
    }
    Ok(RegCodeValidateResp {
        code_type: CodeType::Time,
        expire_time: Some(current_device_expire),
        remaining_count: None,
    })
}
