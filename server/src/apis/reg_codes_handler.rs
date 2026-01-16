use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::{DateTime, FixedOffset, Utc};
use data_model::{app_devices, apps, reg_codes};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use salvo_oapi::{ToSchema, endpoint};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
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
    Used = 1,
    Binded = 2,
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
            1 => RegCodeStatus::Used,
            2 => RegCodeStatus::Binded,
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
    pub max_devices: i32,
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
    pub device_id: Option<i32>,
    pub device_id_str: Option<String>,
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
            device_id: reg_code.device_id,
            device_id_str,
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
            device_id: reg_code.device_id,
            device_id_str: None,
            device_info: None,
            created_at: reg_code.created_at,
            updated_at: reg_code.updated_at,
            app_name: None,
        })
    }
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
    let now = Utc::now().fixed_offset();
    let active_model = reg_codes::ActiveModel {
        code: Set(req.code),
        app_id: Set(req.app_id),
        valid_days: Set(req.valid_days),
        max_devices: Set(req.max_devices),
        status: Set(i16::from(req.status)),
        code_type: Set(i16::from(req.code_type)),
        total_count: Set(req.total_count),
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
        Some((reg_code, app, device)) => Ok(RegCodeInfo::try_from((reg_code, app, device))?),
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
    let reg_code = reg_codes::Entity::find_by_id(id).one(&state.db).await?;
    let reg_code =
        reg_code.ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;

    let mut reg_code: reg_codes::ActiveModel = reg_code.into_active_model();
    if let Some(v) = req.code {
        reg_code.code = Set(v);
    }
    if let Some(v) = req.app_id {
        reg_code.app_id = Set(v);
    }
    if let Some(v) = req.valid_days {
        reg_code.valid_days = Set(v);
    }
    if let Some(v) = req.max_devices {
        reg_code.max_devices = Set(v);
    }
    if let Some(v) = req.status {
        reg_code.status = Set(v);
    }
    if let Some(v) = req.code_type {
        reg_code.code_type = Set(i16::from(v));
    }
    if let Some(v) = req.total_count {
        reg_code.total_count = Set(Some(v));
    }
    reg_code.updated_at = Set(Utc::now().fixed_offset());

    let updated_reg_code = reg_code.update(&state.db).await?;

    // Fetch with app information for response
    let result = reg_codes::Entity::find_by_id(updated_reg_code.id)
        .find_also_related(apps::Entity)
        .find_also_related(app_devices::Entity)
        .one(&state.db)
        .await?;

    match result {
        Some((reg_code, app, device)) => Ok(RegCodeInfo::try_from((reg_code, app, device))?),
        None => Err(AppError::not_found(
            "reg_codes".to_string(),
            Some(updated_reg_code.id),
        )),
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
    let reg_code = reg_code.ok_or_else(|| AppError::not_found("reg_codes".to_string(), Some(id)))?;

    if RegCodeStatus::from(reg_code.status) == RegCodeStatus::Binded {
        return Err(AppError::business_logic(
            "REG_CODE_STATUS_LOCKED",
            "reg code is binded, status cannot be changed",
        ));
    }

    if req.status == RegCodeStatus::Binded {
        return Err(AppError::validation("cannot set reg_code status to binded"));
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
        Some((reg_code, app, device)) => Ok(RegCodeInfo::try_from((reg_code, app, device))?),
        None => Err(AppError::not_found(
            "reg_codes".to_string(),
            Some(updated_reg_code.id),
        )),
    }
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
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

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

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let results = paginator.fetch_page(page - 1).await?;

    let list: Result<Vec<RegCodeInfo>, AppError> = results
        .into_iter()
        .map(|(reg_code, app, device)| RegCodeInfo::try_from((reg_code, app, device)))
        .collect();

    Ok(PagingResponse {
        list: list?,
        total,
        page,
    })
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
        Some((reg_code, app, device)) => Ok(RegCodeInfo::try_from((reg_code, app, device))?),
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
    match app_model.code_type.into() {
        CodeType::Time => {
            time_reg_code_validate(state, &app_model, &device_id, dev, &reg_code_model).await
        }
        CodeType::Count => {
            count_reg_code_validate(state, &app_model, &device_id, dev, &reg_code_model).await
        }
    }
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
                let expire_time =
                    device_expire + chrono::Duration::days(reg_code_model.valid_days as i64);
                let active_device_model = dm.into_active_model();
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

//次数类型注册码验证
pub async fn count_reg_code_validate(
    state: &AppState,
    app_model: &apps::Model,
    device_id: &str,
    device_model: Option<app_devices::Model>,
    reg_code_model: &reg_codes::Model,
) -> Result<RegCodeValidateResp, AppError> {
    let now = Utc::now().fixed_offset();
    if CodeType::from(reg_code_model.code_type) != CodeType::Count {
        return Err(AppError::Message("code type is not count".into()));
    }
    if reg_code_model.device_id.is_some() {
        let device_model =
            device_model.ok_or_else(|| AppError::Message("device not found".into()))?;
        let remaining = device_model.remaining.unwrap_or(0);
        if remaining <= 0 {
            return Err(AppError::Message("device remaining count is 0".into()));
        }
        return Ok(RegCodeValidateResp {
            code_type: CodeType::Count,
            expire_time: None,
            remaining_count: Some(remaining),
        });
    }

    let ctx = state.db.begin().await?;
    let mut reg_code_active = reg_code_model.clone().into_active_model();
    reg_code_active.binding_time = Set(Some(now));
    let remain_count_after = match device_model {
        Some(device_model) => {
            // update device
            let remain_count_before = device_model.remaining.unwrap_or(0);
            if remain_count_before <= 0 {
                return Err(AppError::Message("device remaining count is 0".into()));
            }
            let remain_count_after = remain_count_before - 1;

            reg_code_active.device_id = Set(Some(device_model.id));
            let mut active_device_model = device_model.into_active_model();
            active_device_model.remaining = Set(Some(remain_count_after));
            active_device_model.update(&ctx).await?;
            remain_count_after
        }
        None => {
            // add device
            let remain_count_before = reg_code_model.total_count.unwrap_or(0);
            if remain_count_before <= 0 {
                return Err(AppError::Message("reg code remaining count is 0".into()));
            }
            let remain_count_after = remain_count_before - 1;

            let device_active = app_devices::ActiveModel {
                app_id: Set(app_model.id),
                device_id: Set(device_id.to_string()),
                remaining: Set(Some(remain_count_after)),
                ..Default::default()
            };
            let inserted_device = device_active.insert(&ctx).await?;
            reg_code_active.device_id = Set(Some(inserted_device.id));
            remain_count_after
        }
    };
    reg_code_active.update(&ctx).await?;
    ctx.commit().await?;
    return Ok(RegCodeValidateResp {
        code_type: CodeType::Count,
        expire_time: None,
        remaining_count: Some(remain_count_after),
    });
}
