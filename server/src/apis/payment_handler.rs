use crate::apis::auth_middleware::Claims;
use crate::apis::auth_middleware::optional_claims;
use crate::apis::distribution_handler::{handle_commission_refund, new_commission_active_model};
use crate::apis::email_verification_handler::normalize_email;
use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::apis::reg_codes_handler::{CodeType, RegCodeStatus, revoke_reg_code_for_order};
use crate::apis::system_settings_handler::get_distribution_settings;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::resource_service::{ResourceUpload, download_resource, upload_resource};
use crate::core::response::ApiResponse;
use chrono::Utc;
use data_model::{
    apps, distribution_commissions, license_plans, order_events, order_refund_attachments,
    order_refunds, orders, payment_channels, reg_codes, resources, users,
};
use payment_adapter::{
    AlipayPageAdapter, AlipayPageConfig, CreatePaymentRequest, PaymentAdapter, PaymentError,
    PaymentHeaders, PaymentNotification, PaymentStatus, WechatNativeAdapter, WechatNativeConfig,
};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Set,
    Statement, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;
use validator::Validate;

const PROVIDER_WECHAT: &str = "wechat";
const PROVIDER_ALIPAY: &str = "alipay";
const PAY_TYPE_WECHAT_NATIVE: &str = "wechat_native";
const PAY_TYPE_ALIPAY: &str = "alipay";
const DEFAULT_WECHAT_API_BASE_URL: &str = "https://api.mch.weixin.qq.com";
const DEFAULT_ALIPAY_GATEWAY_URL: &str = "https://openapi.alipay.com/gateway.do";
const ORDER_EVENT_PAYMENT_DELIVERED: &str = "payment.delivered";
const ORDER_EVENT_REFUND_CONFIRMED: &str = "refund.confirmed";
const ORDER_EVENTS_NOTIFY_CHANNEL: &str = "licensehub_order_events";
const APP_STATUS_ENABLED: i16 = 1;
const REFUND_STATUS_SUCCEEDED: i16 = 1;
const MAX_REFUND_ATTACHMENT_SIZE: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PlanStatus {
    Disabled = 0,
    Enabled = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PaymentChannelStatus {
    Disabled = 0,
    Enabled = 1,
}

impl From<i16> for PaymentChannelStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => PaymentChannelStatus::Enabled,
            _ => PaymentChannelStatus::Disabled,
        }
    }
}

impl From<PaymentChannelStatus> for i16 {
    fn from(value: PaymentChannelStatus) -> Self {
        value as i16
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatChannelConfig {
    pub app_id: String,
    pub mch_id: String,
    pub merchant_serial_no: String,
    pub merchant_private_key_pem: String,
    pub api_v3_key: String,
    pub wechatpay_public_key_id: String,
    pub wechatpay_public_key_pem: String,
    pub api_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlipayChannelConfig {
    pub app_id: String,
    pub app_private_key_pem: String,
    pub alipay_public_key_pem: String,
    pub gateway_url: String,
    pub seller_id: String,
}

impl From<i16> for PlanStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => PlanStatus::Enabled,
            _ => PlanStatus::Disabled,
        }
    }
}

impl From<PlanStatus> for i16 {
    fn from(value: PlanStatus) -> Self {
        value as i16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum OrderStatus {
    Pending = 0,
    Paid = 1,
    Delivered = 2,
    Failed = 3,
    Closed = 4,
    Refunded = 5,
}

impl From<i16> for OrderStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => OrderStatus::Paid,
            2 => OrderStatus::Delivered,
            3 => OrderStatus::Failed,
            4 => OrderStatus::Closed,
            5 => OrderStatus::Refunded,
            _ => OrderStatus::Pending,
        }
    }
}

impl From<OrderStatus> for i16 {
    fn from(value: OrderStatus) -> Self {
        value as i16
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePlanReq {
    pub app_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub code_type: CodeType,
    pub valid_days: Option<i32>,
    pub total_count: Option<i32>,
    pub status: PlanStatus,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePlanReq {
    pub app_id: Option<i32>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_cents: Option<i32>,
    pub code_type: Option<CodeType>,
    pub valid_days: Option<i32>,
    pub total_count: Option<i32>,
    pub status: Option<PlanStatus>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListPlansParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub id: Option<i32>,
    pub app_id: Option<i32>,
    pub name: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PublicPlansParams {
    pub app_id: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicPlansState {
    Available,
    AppDisabled,
    AppNotFound,
}

#[derive(Debug, Serialize)]
pub struct PublicPlansInfo {
    pub state: PublicPlansState,
    pub app_id: Option<i32>,
    pub app_name: Option<String>,
    pub app_website_url: Option<String>,
    pub app_status: Option<i16>,
    pub plans: Vec<PlanInfo>,
}

#[derive(Debug, Serialize)]
pub struct PlanInfo {
    pub id: i32,
    pub app_id: i32,
    pub app_name: Option<String>,
    pub app_website_url: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i32,
    pub code_type: CodeType,
    pub valid_days: i32,
    pub total_count: Option<i32>,
    pub status: PlanStatus,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<(license_plans::Model, Option<apps::Model>)> for PlanInfo {
    fn from(value: (license_plans::Model, Option<apps::Model>)) -> Self {
        let (plan, app) = value;
        let app_name = app.as_ref().map(|app| app.name.clone());
        let app_website_url = app.and_then(|app| app.website_url);
        Self {
            id: plan.id,
            app_id: plan.app_id,
            app_name,
            app_website_url,
            name: plan.name,
            description: plan.description,
            price_cents: plan.price_cents,
            code_type: CodeType::from(plan.code_type),
            valid_days: plan.valid_days,
            total_count: plan.total_count,
            status: PlanStatus::from(plan.status),
            sort_order: plan.sort_order,
            created_at: plan.created_at,
            updated_at: plan.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentChannelReq {
    pub name: String,
    pub provider: String,
    pub pay_type: String,
    pub status: PaymentChannelStatus,
    pub sort_order: Option<i32>,
    pub config: Value,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaymentChannelReq {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub pay_type: Option<String>,
    pub status: Option<PaymentChannelStatus>,
    pub sort_order: Option<i32>,
    pub config: Option<Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListPaymentChannelsParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub id: Option<i32>,
    pub provider: Option<String>,
    pub pay_type: Option<String>,
    pub status: Option<i16>,
}

#[derive(Debug, Serialize)]
pub struct PaymentChannelInfo {
    pub id: i32,
    pub name: String,
    pub provider: String,
    pub pay_type: String,
    pub status: PaymentChannelStatus,
    pub sort_order: i32,
    pub config: Value,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl TryFrom<payment_channels::Model> for PaymentChannelInfo {
    type Error = AppError;

    fn try_from(channel: payment_channels::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: channel.id,
            name: channel.name,
            provider: channel.provider.clone(),
            pay_type: channel.pay_type,
            status: PaymentChannelStatus::from(channel.status),
            sort_order: channel.sort_order,
            config: response_channel_config(&channel.provider, &channel.config)?,
            created_at: channel.created_at,
            updated_at: channel.updated_at,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderReq {
    pub plan_id: i32,
    pub pay_type: String,
    pub referral_code: Option<String>,
    pub buyer_email: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListOrdersParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub order_no: Option<String>,
    pub reg_code: Option<String>,
    pub buyer: Option<String>,
    pub status: Option<i16>,
    pub plan_id: Option<i32>,
    pub app_id: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmOrderRefundReq {
    #[validate(length(min = 1, max = 255))]
    pub refund_reference: String,
    #[validate(length(min = 1, max = 1000))]
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct OrderRefundInfo {
    pub refund_no: String,
    pub refund_reference: String,
    pub reason: String,
    pub operator_user_id: i32,
    pub attachment_file_name: Option<String>,
    pub attachment_content_type: Option<String>,
    pub attachment_size: Option<i64>,
    pub refunded_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct OrderInfo {
    pub id: i32,
    pub order_no: String,
    pub plan_id: i32,
    pub plan_name: Option<String>,
    pub app_id: i32,
    pub app_name: Option<String>,
    pub amount_cents: i32,
    pub pay_type: String,
    pub status: OrderStatus,
    pub provider: String,
    pub provider_trade_no: Option<String>,
    pub pay_url: Option<String>,
    pub qr_code: Option<String>,
    pub url_scheme: Option<String>,
    pub reg_code_id: Option<i32>,
    pub reg_code: Option<String>,
    pub buyer_user_id: Option<i32>,
    pub buyer_email: Option<String>,
    pub referrer_user_id: Option<i32>,
    pub referral_code: Option<String>,
    pub commission_rate_bps: Option<i32>,
    pub commission_amount_cents: Option<i32>,
    pub refund: Option<OrderRefundInfo>,
    pub paid_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct PublicOrderInfo {
    pub id: i32,
    pub order_no: String,
    pub plan_id: i32,
    pub plan_name: Option<String>,
    pub app_id: i32,
    pub app_name: Option<String>,
    pub amount_cents: i32,
    pub pay_type: String,
    pub status: OrderStatus,
    pub provider: String,
    pub provider_trade_no: Option<String>,
    pub reg_code: Option<String>,
    pub paid_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

impl From<OrderInfo> for PublicOrderInfo {
    fn from(order: OrderInfo) -> Self {
        Self {
            id: order.id,
            order_no: order.order_no,
            plan_id: order.plan_id,
            plan_name: order.plan_name,
            app_id: order.app_id,
            app_name: order.app_name,
            amount_cents: order.amount_cents,
            pay_type: order.pay_type,
            status: order.status,
            provider: order.provider,
            provider_trade_no: order.provider_trade_no,
            reg_code: order.reg_code,
            paid_at: order.paid_at,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

impl
    From<(
        orders::Model,
        Option<license_plans::Model>,
        Option<apps::Model>,
    )> for OrderInfo
{
    fn from(
        value: (
            orders::Model,
            Option<license_plans::Model>,
            Option<apps::Model>,
        ),
    ) -> Self {
        let (order, plan, app) = value;
        Self {
            id: order.id,
            order_no: order.order_no,
            plan_id: order.plan_id,
            plan_name: plan.map(|p| p.name),
            app_id: order.app_id,
            app_name: app.map(|a| a.name),
            amount_cents: order.amount_cents,
            pay_type: order.pay_type,
            status: OrderStatus::from(order.status),
            provider: order.provider,
            provider_trade_no: order.provider_trade_no,
            pay_url: order.pay_url,
            qr_code: order.qr_code,
            url_scheme: order.url_scheme,
            reg_code_id: order.reg_code_id,
            reg_code: None,
            buyer_user_id: order.buyer_user_id,
            buyer_email: order.buyer_email,
            referrer_user_id: order.referrer_user_id,
            referral_code: order.referral_code,
            commission_rate_bps: order.commission_rate_bps,
            commission_amount_cents: order.commission_amount_cents,
            refund: None,
            paid_at: order.paid_at,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PayMethodInfo {
    pub pay_type: String,
    pub label: String,
    pub provider: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct PayMethodsInfo {
    pub enabled: bool,
    pub provider: String,
    pub merchant_active: bool,
    pub methods: Vec<PayMethodInfo>,
    pub message: Option<String>,
}

#[handler]
pub async fn list_public_plans(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PublicPlansInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<PublicPlansParams>().unwrap_or_default();
    let app = match params.app_id {
        Some(app_id) => match apps::Entity::find_by_id(app_id).one(&state.db).await? {
            Some(app) if app.status != APP_STATUS_ENABLED => {
                return Ok(ApiResponse::success(PublicPlansInfo {
                    state: PublicPlansState::AppDisabled,
                    app_id: Some(app.id),
                    app_name: Some(app.name),
                    app_website_url: app.website_url,
                    app_status: Some(app.status),
                    plans: Vec::new(),
                }));
            }
            Some(app) => Some(app),
            None => {
                return Ok(ApiResponse::success(PublicPlansInfo {
                    state: PublicPlansState::AppNotFound,
                    app_id: Some(app_id),
                    app_name: None,
                    app_website_url: None,
                    app_status: None,
                    plans: Vec::new(),
                }));
            }
        },
        None => None,
    };
    let mut query = license_plans::Entity::find()
        .find_also_related(apps::Entity)
        .filter(license_plans::Column::Status.eq(i16::from(PlanStatus::Enabled)))
        .filter(apps::Column::Status.eq(APP_STATUS_ENABLED))
        .order_by_asc(license_plans::Column::SortOrder)
        .order_by_asc(license_plans::Column::Id);
    if let Some(app_id) = params.app_id {
        query = query.filter(license_plans::Column::AppId.eq(app_id));
    }
    let rows = query.all(&state.db).await?;
    Ok(ApiResponse::success(PublicPlansInfo {
        state: PublicPlansState::Available,
        app_id: app.as_ref().map(|app| app.id),
        app_name: app.as_ref().map(|app| app.name.clone()),
        app_website_url: app.as_ref().and_then(|app| app.website_url.clone()),
        app_status: app.as_ref().map(|app| app.status),
        plans: rows.into_iter().map(PlanInfo::from).collect(),
    }))
}

#[handler]
pub async fn list_pay_methods(depot: &mut Depot) -> Result<ApiResponse<PayMethodsInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    Ok(ApiResponse::success(fetch_pay_methods_impl(state).await?))
}

#[handler]
pub async fn create_plan(
    depot: &mut Depot,
    req: JsonBody<CreatePlanReq>,
) -> Result<ApiResponse<PlanInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let req = req.into_inner();
    let plan = create_plan_impl(state, req).await?;
    Ok(ApiResponse::success(plan))
}

pub async fn create_plan_impl(state: &AppState, req: CreatePlanReq) -> Result<PlanInfo, AppError> {
    ensure_plan_matches_app(state, req.app_id, req.code_type).await?;
    validate_plan_input(
        req.price_cents,
        req.code_type,
        req.valid_days.unwrap_or_default(),
        req.total_count,
    )?;
    let now = Utc::now().fixed_offset();
    let valid_days = match req.code_type {
        CodeType::Time => req.valid_days.unwrap_or_default(),
        CodeType::Count => 0,
    };
    let total_count = match req.code_type {
        CodeType::Time => None,
        CodeType::Count => req.total_count,
    };
    let active = license_plans::ActiveModel {
        app_id: Set(req.app_id),
        name: Set(req.name),
        description: Set(req.description),
        price_cents: Set(req.price_cents),
        code_type: Set(i16::from(req.code_type)),
        valid_days: Set(valid_days),
        total_count: Set(total_count),
        status: Set(i16::from(req.status)),
        sort_order: Set(req.sort_order.unwrap_or_default()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let plan = active.insert(&state.db).await?;
    get_plan_by_id_impl(state, plan.id).await
}

#[handler]
pub async fn update_plan(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UpdatePlanReq>,
) -> Result<ApiResponse<PlanInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let plan = update_plan_impl(state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(plan))
}

pub async fn update_plan_impl(
    state: &AppState,
    id: i32,
    req: UpdatePlanReq,
) -> Result<PlanInfo, AppError> {
    let plan = license_plans::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(id)))?;
    let final_app_id = req.app_id.unwrap_or(plan.app_id);
    let final_price = req.price_cents.unwrap_or(plan.price_cents);
    let final_type = req
        .code_type
        .unwrap_or_else(|| CodeType::from(plan.code_type));
    let final_days = req.valid_days.unwrap_or(plan.valid_days);
    let final_total = req.total_count.or(plan.total_count);
    ensure_plan_matches_app(state, final_app_id, final_type).await?;
    validate_plan_input(final_price, final_type, final_days, final_total)?;

    let mut active = plan.into_active_model();
    if let Some(v) = req.app_id {
        active.app_id = Set(v);
    }
    if let Some(v) = req.name {
        active.name = Set(v);
    }
    if let Some(v) = req.description {
        active.description = Set(Some(v));
    }
    if let Some(v) = req.price_cents {
        active.price_cents = Set(v);
    }
    if let Some(v) = req.code_type {
        active.code_type = Set(i16::from(v));
    }
    if let Some(v) = req.valid_days {
        active.valid_days = Set(v);
    }
    if let Some(v) = req.total_count {
        active.total_count = Set(Some(v));
    }
    match final_type {
        CodeType::Time => {
            active.total_count = Set(None);
        }
        CodeType::Count => {
            active.valid_days = Set(0);
        }
    }
    if let Some(v) = req.status {
        active.status = Set(i16::from(v));
    }
    if let Some(v) = req.sort_order {
        active.sort_order = Set(v);
    }
    active.updated_at = Set(Utc::now().fixed_offset());
    let updated = active.update(&state.db).await?;
    get_plan_by_id_impl(state, updated.id).await
}

#[handler]
pub async fn create_payment_channel(
    depot: &mut Depot,
    req: JsonBody<CreatePaymentChannelReq>,
) -> Result<ApiResponse<PaymentChannelInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let channel = create_payment_channel_impl(state, req.into_inner()).await?;
    Ok(ApiResponse::success(channel))
}

pub async fn create_payment_channel_impl(
    state: &AppState,
    req: CreatePaymentChannelReq,
) -> Result<PaymentChannelInfo, AppError> {
    let provider = normalize_provider(&req.provider)?;
    let pay_type = normalize_pay_type(&req.pay_type)?;
    let name = normalize_required_text(req.name, "name")?;
    let config = normalize_channel_config(&provider, &req.config)?;
    let now = Utc::now().fixed_offset();
    let active = payment_channels::ActiveModel {
        name: Set(name),
        provider: Set(provider),
        pay_type: Set(pay_type),
        status: Set(i16::from(req.status)),
        sort_order: Set(req.sort_order.unwrap_or_default()),
        config: Set(config),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let channel = active.insert(&state.db).await?;
    get_payment_channel_by_id_impl(state, channel.id).await
}

#[handler]
pub async fn update_payment_channel(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UpdatePaymentChannelReq>,
) -> Result<ApiResponse<PaymentChannelInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let channel = update_payment_channel_impl(state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(channel))
}

pub async fn update_payment_channel_impl(
    state: &AppState,
    id: i32,
    req: UpdatePaymentChannelReq,
) -> Result<PaymentChannelInfo, AppError> {
    let channel = payment_channels::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("payment_channels", Some(id)))?;
    let final_provider = match req.provider.as_deref() {
        Some(provider) => normalize_provider(provider)?,
        None => channel.provider.clone(),
    };
    let final_pay_type = match req.pay_type.as_deref() {
        Some(pay_type) => normalize_pay_type(pay_type)?,
        None => channel.pay_type.clone(),
    };
    let final_config = match req.config.as_ref() {
        Some(config) => normalize_channel_config(&final_provider, config)?,
        None => normalize_channel_config(&final_provider, &channel.config)?,
    };

    let mut active = channel.into_active_model();
    if let Some(name) = req.name {
        active.name = Set(normalize_required_text(name, "name")?);
    }
    active.provider = Set(final_provider);
    active.pay_type = Set(final_pay_type);
    if let Some(status) = req.status {
        active.status = Set(i16::from(status));
    }
    if let Some(sort_order) = req.sort_order {
        active.sort_order = Set(sort_order);
    }
    active.config = Set(final_config);
    active.updated_at = Set(Utc::now().fixed_offset());
    let updated = active.update(&state.db).await?;
    get_payment_channel_by_id_impl(state, updated.id).await
}

#[handler]
pub async fn delete_payment_channel(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let id = id.into_inner();
    let channel = payment_channels::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("payment_channels", Some(id)))?;
    channel.into_active_model().delete(&state.db).await?;
    Ok(ApiResponse::success(()))
}

#[handler]
pub async fn list_payment_channels(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<PaymentChannelInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListPaymentChannelsParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = payment_channels::Entity::find()
        .order_by_asc(payment_channels::Column::SortOrder)
        .order_by_asc(payment_channels::Column::Id);
    if let Some(id) = params.id {
        query = query.filter(payment_channels::Column::Id.eq(id));
    }
    if let Some(provider) = params.provider {
        query = query.filter(payment_channels::Column::Provider.eq(normalize_provider(&provider)?));
    }
    if let Some(pay_type) = params.pay_type {
        query = query.filter(payment_channels::Column::PayType.eq(normalize_pay_type(&pay_type)?));
    }
    if let Some(status) = params.status {
        validate_channel_status(status)?;
        query = query.filter(payment_channels::Column::Status.eq(status));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let list = rows
        .into_iter()
        .map(PaymentChannelInfo::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ApiResponse::success(PagingResponse { list, total, page }))
}

pub async fn get_payment_channel_by_id_impl(
    state: &AppState,
    id: i32,
) -> Result<PaymentChannelInfo, AppError> {
    let channel = payment_channels::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("payment_channels", Some(id)))?;
    PaymentChannelInfo::try_from(channel)
}

async fn fetch_pay_methods_impl(state: &AppState) -> Result<PayMethodsInfo, AppError> {
    let cfg = &state.config.payment;
    if !cfg.enabled {
        return Ok(PayMethodsInfo {
            enabled: false,
            provider: "database".to_string(),
            merchant_active: false,
            methods: Vec::new(),
            message: Some("payment is disabled".to_string()),
        });
    }
    let channels = payment_channels::Entity::find()
        .filter(payment_channels::Column::Status.eq(i16::from(PaymentChannelStatus::Enabled)))
        .order_by_asc(payment_channels::Column::SortOrder)
        .order_by_asc(payment_channels::Column::Id)
        .all(&state.db)
        .await?;
    if channels.is_empty() {
        return Ok(PayMethodsInfo {
            enabled: false,
            provider: "database".to_string(),
            merchant_active: false,
            methods: Vec::new(),
            message: Some("no payment channel is configured".to_string()),
        });
    }

    let methods = channels
        .into_iter()
        .map(|channel| PayMethodInfo {
            pay_type: channel.pay_type,
            label: channel.name,
            provider: channel.provider,
            enabled: true,
        })
        .collect::<Vec<_>>();
    Ok(PayMethodsInfo {
        enabled: !methods.is_empty(),
        provider: "database".to_string(),
        merchant_active: true,
        methods,
        message: None,
    })
}

#[handler]
pub async fn delete_plan(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let id = id.into_inner();
    let plan = license_plans::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(id)))?;
    plan.into_active_model().delete(&state.db).await?;
    Ok(ApiResponse::success(()))
}

#[handler]
pub async fn list_plans(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<PlanInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListPlansParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = license_plans::Entity::find()
        .find_also_related(apps::Entity)
        .order_by_asc(license_plans::Column::SortOrder)
        .order_by_asc(license_plans::Column::Id);
    if let Some(id) = params.id {
        query = query.filter(license_plans::Column::Id.eq(id));
    }
    if let Some(app_id) = params.app_id {
        query = query.filter(license_plans::Column::AppId.eq(app_id));
    }
    if let Some(name) = params.name {
        query = query.filter(license_plans::Column::Name.contains(name));
    }
    if let Some(status) = params.status {
        query = query.filter(license_plans::Column::Status.eq(status));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    Ok(ApiResponse::success(PagingResponse {
        list: rows.into_iter().map(PlanInfo::from).collect(),
        total,
        page,
    }))
}

pub async fn get_plan_by_id_impl(state: &AppState, id: i32) -> Result<PlanInfo, AppError> {
    let row = license_plans::Entity::find_by_id(id)
        .find_also_related(apps::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(id)))?;
    Ok(PlanInfo::from(row))
}

#[handler]
pub async fn create_order(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<OrderInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let body = req.parse_json::<CreateOrderReq>().await?;
    let buyer_user_id = optional_claims(req, state)
        .map_err(|_| AppError::auth_failed("登录状态无效或已过期"))?
        .map(|claims| claims.user_id);
    let client_ip = req.remote_addr().to_string();
    let order = create_order_impl(state, body, Some(client_ip), buyer_user_id).await?;
    Ok(ApiResponse::success(order))
}

pub async fn create_order_impl(
    state: &AppState,
    req: CreateOrderReq,
    client_ip: Option<String>,
    buyer_user_id: Option<i32>,
) -> Result<OrderInfo, AppError> {
    let pay_type = normalize_pay_type(&req.pay_type)?;
    let channel = find_payment_channel_by_pay_type(state, &pay_type).await?;
    let provider = channel
        .as_ref()
        .map(|channel| channel.provider.as_str())
        .map(Ok)
        .unwrap_or_else(|| provider_for_pay_type(&pay_type))?
        .to_string();
    let (plan, app) = license_plans::Entity::find_by_id(req.plan_id)
        .find_also_related(apps::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(req.plan_id)))?;
    if PlanStatus::from(plan.status) != PlanStatus::Enabled {
        return Err(AppError::business_logic(
            "PLAN_DISABLED",
            "plan is disabled",
        ));
    }
    let app = app.ok_or_else(|| AppError::not_found("apps", Some(plan.app_id)))?;
    if app.status != APP_STATUS_ENABLED {
        return Err(AppError::business_logic("APP_DISABLED", "app is disabled"));
    }
    if plan.price_cents <= 0 {
        return Err(AppError::business_logic(
            "INVALID_PRICE",
            "plan price is invalid",
        ));
    }

    let buyer = if let Some(user_id) = buyer_user_id {
        let user = users::Entity::find_by_id(user_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::auth_failed("用户不存在"))?;
        (Some(user.id), user.email, user.referrer_user_id)
    } else {
        let email = req
            .buyer_email
            .as_deref()
            .ok_or_else(|| AppError::business_logic("BUYER_EMAIL_REQUIRED", "游客购买需要填写邮箱"))
            .and_then(normalize_email)?;
        (None, Some(email), None)
    };

    let now = Utc::now().fixed_offset();
    let order_no = new_order_no();
    let distribution = get_distribution_settings(state).await?;
    let attribution = if distribution.enabled {
        let referrer = if let Some(referrer_user_id) = buyer.2 {
            users::Entity::find_by_id(referrer_user_id)
                .one(&state.db)
                .await?
        } else if buyer.0.is_none() {
            if let Some(code) = req
                .referral_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                users::Entity::find()
                    .filter(users::Column::ReferralCode.eq(code.to_ascii_uppercase()))
                    .one(&state.db)
                    .await?
            } else {
                None
            }
        } else {
            None
        };
        referrer.and_then(|user| {
            if Some(user.id) == buyer.0 {
                return None;
            }
            let rate = user
                .commission_rate_bps
                .unwrap_or(distribution.default_rate_bps);
            let amount = ((plan.price_cents as i64 * rate as i64) / 10000) as i32;
            Some((user.id, user.referral_code, rate, amount))
        })
    } else {
        None
    };
    let mut active = orders::ActiveModel {
        order_no: Set(order_no.clone()),
        plan_id: Set(plan.id),
        app_id: Set(plan.app_id),
        amount_cents: Set(plan.price_cents),
        pay_type: Set(pay_type.clone()),
        status: Set(i16::from(OrderStatus::Pending)),
        provider: Set(provider.clone()),
        client_ip: Set(client_ip.clone()),
        buyer_user_id: Set(buyer.0),
        buyer_email: Set(buyer.1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    if let Some((user_id, code, rate, amount)) = attribution {
        active.referrer_user_id = Set(Some(user_id));
        active.referral_code = Set(Some(code));
        active.commission_rate_bps = Set(Some(rate));
        active.commission_amount_cents = Set(Some(amount));
    }

    if state.config.payment.enabled {
        let channel = channel
            .filter(|channel| {
                PaymentChannelStatus::from(channel.status) == PaymentChannelStatus::Enabled
            })
            .ok_or_else(|| {
                AppError::business_logic(
                    "PAYMENT_CHANNEL_UNAVAILABLE",
                    "payment channel is not configured or disabled",
                )
            })?;
        let adapter = build_payment_adapter(&channel)?;
        let notify_url = notify_url_for_pay_type(state, &channel.pay_type)?;
        let pay_resp = adapter
            .create_payment(CreatePaymentRequest {
                out_trade_no: order_no.clone(),
                subject: plan.name.clone(),
                amount_cents: plan.price_cents,
                notify_url: notify_url.clone(),
                return_url: return_url_for_provider(state, &channel.provider, &order_no),
                client_ip,
                attach: Some(plan.id.to_string()),
            })
            .await
            .map_err(payment_error)?;
        active.provider = Set(channel.provider.clone());
        active.pay_type = Set(channel.pay_type.clone());
        active.provider_trade_no = Set(pay_resp.provider_trade_no);
        active.pay_url = Set(pay_resp.pay_url);
        active.qr_code = Set(pay_resp.qr_code);
        active.url_scheme = Set(pay_resp.url_scheme);
        active.provider_payload = Set(Some(json!({
            "channel_id": channel.id,
            "provider": channel.provider,
            "pay_type": channel.pay_type,
            "notify_url": notify_url,
            "adapter": pay_resp.raw_payload,
        })));
    }

    let inserted = active.insert(&state.db).await?;
    get_order_by_no_impl(state, &inserted.order_no).await
}

#[handler]
pub async fn get_order(
    depot: &mut Depot,
    order_no: PathParam<String>,
) -> Result<ApiResponse<PublicOrderInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let order_no = order_no.into_inner();
    let order = get_order_by_no_impl(state, &order_no).await?;
    let order = sync_pending_order_from_provider(state, order).await?;
    Ok(ApiResponse::success(PublicOrderInfo::from(order)))
}

#[handler]
pub async fn list_orders(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<OrderInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListOrdersParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = orders::Entity::find()
        .find_also_related(license_plans::Entity)
        .find_also_related(apps::Entity)
        .order_by_desc(orders::Column::CreatedAt);
    if let Some(order_no) = params.order_no {
        query = query.filter(orders::Column::OrderNo.contains(order_no));
    }
    if let Some(reg_code) = params.reg_code {
        query = query
            .join(
                sea_orm::JoinType::InnerJoin,
                orders::Relation::RegCodes.def(),
            )
            .filter(reg_codes::Column::Code.contains(reg_code));
    }
    if let Some(buyer) = params.buyer {
        let buyer = buyer.trim().to_string();
        if !buyer.is_empty() {
            let mut condition = Condition::any()
                .add(orders::Column::BuyerEmail.contains(buyer.clone()))
                .add(users::Column::Username.contains(buyer.clone()))
                .add(users::Column::Email.contains(buyer.clone()));
            if let Ok(user_id) = buyer.parse::<i32>() {
                condition = condition.add(orders::Column::BuyerUserId.eq(user_id));
            }
            query = query
                .join(
                    sea_orm::JoinType::LeftJoin,
                    orders::Relation::BuyerUser.def(),
                )
                .filter(condition);
        }
    }
    if let Some(status) = params.status {
        query = query.filter(orders::Column::Status.eq(status));
    }
    if let Some(plan_id) = params.plan_id {
        query = query.filter(orders::Column::PlanId.eq(plan_id));
    }
    if let Some(app_id) = params.app_id {
        query = query.filter(orders::Column::AppId.eq(app_id));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let mut list = Vec::new();
    for row in rows {
        list.push(build_order_info(state, row).await?);
    }
    Ok(ApiResponse::success(PagingResponse { list, total, page }))
}

#[handler]
pub async fn list_my_orders(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<OrderInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let params = req.parse_queries::<ListOrdersParams>()?;
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = orders::Entity::find()
        .filter(orders::Column::BuyerUserId.eq(claims.user_id))
        .find_also_related(license_plans::Entity)
        .find_also_related(apps::Entity)
        .order_by_desc(orders::Column::CreatedAt);
    if let Some(order_no) = params.order_no {
        query = query.filter(orders::Column::OrderNo.contains(order_no));
    }
    if let Some(reg_code) = params.reg_code {
        query = query
            .join(
                sea_orm::JoinType::InnerJoin,
                orders::Relation::RegCodes.def(),
            )
            .filter(reg_codes::Column::Code.contains(reg_code));
    }
    if let Some(buyer) = params.buyer {
        let buyer = buyer.trim().to_string();
        if !buyer.is_empty() {
            let mut condition = Condition::any()
                .add(orders::Column::BuyerEmail.contains(buyer.clone()))
                .add(users::Column::Username.contains(buyer.clone()))
                .add(users::Column::Email.contains(buyer.clone()));
            if let Ok(user_id) = buyer.parse::<i32>() {
                condition = condition.add(orders::Column::BuyerUserId.eq(user_id));
            }
            query = query
                .join(
                    sea_orm::JoinType::LeftJoin,
                    orders::Relation::BuyerUser.def(),
                )
                .filter(condition);
        }
    }
    if let Some(status) = params.status {
        query = query.filter(orders::Column::Status.eq(status));
    }
    if let Some(plan_id) = params.plan_id {
        query = query.filter(orders::Column::PlanId.eq(plan_id));
    }
    if let Some(app_id) = params.app_id {
        query = query.filter(orders::Column::AppId.eq(app_id));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let mut list = Vec::with_capacity(rows.len());
    for row in rows {
        list.push(build_order_info(state, row).await?);
    }
    Ok(ApiResponse::success(PagingResponse { list, total, page }))
}

#[handler]
pub async fn get_my_order(
    depot: &mut Depot,
    order_no: PathParam<String>,
) -> Result<ApiResponse<OrderInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let row = orders::Entity::find()
        .filter(orders::Column::OrderNo.eq(order_no.into_inner()))
        .filter(orders::Column::BuyerUserId.eq(claims.user_id))
        .find_also_related(license_plans::Entity)
        .find_also_related(apps::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("orders", None))?;
    Ok(ApiResponse::success(build_order_info(state, row).await?))
}

#[handler]
pub async fn confirm_order_refund(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: &mut Request,
) -> Result<ApiResponse<OrderInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (req, attachment) = if content_type.starts_with("multipart/form-data") {
        let form = req.form_data().await?;
        let refund_reference = form
            .fields
            .get("refund_reference")
            .cloned()
            .unwrap_or_default();
        let reason = form.fields.get("reason").cloned().unwrap_or_default();
        let attachment = if let Some(file) = form.files.get("attachment") {
            if file.size() > MAX_REFUND_ATTACHMENT_SIZE {
                return Err(AppError::validation(
                    "refund attachment must not exceed 5 MB",
                ));
            }
            Some(RefundAttachment {
                file_name: file.name().unwrap_or("refund-attachment").to_string(),
                content_type: file
                    .content_type()
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
                content: tokio::fs::read(file.path()).await.map_err(|error| {
                    AppError::InternalError {
                        message: format!("failed to read refund attachment: {error}"),
                    }
                })?,
            })
        } else {
            None
        };
        (
            ConfirmOrderRefundReq {
                refund_reference,
                reason,
            },
            attachment,
        )
    } else {
        (req.parse_json::<ConfirmOrderRefundReq>().await?, None)
    };
    req.validate()?;
    Ok(ApiResponse::success(
        confirm_order_refund_with_attachment_impl(
            state,
            claims.user_id,
            id.into_inner(),
            req,
            attachment,
        )
        .await?,
    ))
}

#[derive(Debug)]
struct RefundAttachment {
    file_name: String,
    content_type: String,
    content: Vec<u8>,
}

fn validate_refund_attachment(attachment: RefundAttachment) -> Result<RefundAttachment, AppError> {
    if attachment.content.is_empty() || attachment.content.len() as u64 > MAX_REFUND_ATTACHMENT_SIZE
    {
        return Err(AppError::validation(
            "refund attachment must be between 1 byte and 5 MB",
        ));
    }
    let content_type = attachment.content_type.to_ascii_lowercase();
    if !matches!(
        content_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp"
    ) {
        return Err(AppError::validation(
            "refund attachment must be JPG, PNG, or WebP",
        ));
    }
    let file_name = std::path::Path::new(&attachment.file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("refund-attachment")
        .chars()
        .take(255)
        .collect::<String>();
    Ok(RefundAttachment {
        file_name,
        content_type,
        content: attachment.content,
    })
}

pub async fn confirm_order_refund_impl(
    state: &AppState,
    operator_user_id: i32,
    order_id: i32,
    req: ConfirmOrderRefundReq,
) -> Result<OrderInfo, AppError> {
    confirm_order_refund_with_attachment_impl(state, operator_user_id, order_id, req, None).await
}

async fn confirm_order_refund_with_attachment_impl(
    state: &AppState,
    operator_user_id: i32,
    order_id: i32,
    req: ConfirmOrderRefundReq,
    attachment: Option<RefundAttachment>,
) -> Result<OrderInfo, AppError> {
    let refund_reference = req.refund_reference.trim().to_string();
    let reason = req.reason.trim().to_string();
    if refund_reference.is_empty() || reason.is_empty() {
        return Err(AppError::validation(
            "refund_reference and reason must not be empty",
        ));
    }
    let attachment = attachment.map(validate_refund_attachment).transpose()?;

    let tx = state.db.begin().await?;
    let order = orders::Entity::find_by_id(order_id)
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("orders", Some(order_id)))?;

    match OrderStatus::from(order.status) {
        OrderStatus::Refunded => {
            tx.commit().await?;
            return get_order_by_no_impl(state, &order.order_no).await;
        }
        OrderStatus::Delivered => {}
        _ => {
            return Err(AppError::business_logic(
                "ORDER_REFUND_FORBIDDEN",
                "only delivered orders can be confirmed as refunded",
            ));
        }
    }

    if let Some(commission) = distribution_commissions::Entity::find()
        .filter(distribution_commissions::Column::OrderId.eq(order.id))
        .one(&tx)
        .await?
    {
        handle_commission_refund(&tx, commission, operator_user_id).await?;
    }

    if let Some(reg_code_id) = order.reg_code_id {
        revoke_reg_code_for_order(&tx, reg_code_id).await?;
    }

    let now = Utc::now().fixed_offset();
    let refund_no = format!("RF{}", Uuid::new_v4().simple());
    let resource = if let Some(attachment) = attachment.as_ref() {
        Some(
            upload_resource(
                state,
                &tx,
                operator_user_id,
                ResourceUpload {
                    resource_type: "refund_attachment".to_string(),
                    original_name: attachment.file_name.clone(),
                    content_type: attachment.content_type.clone(),
                    content: attachment.content.clone(),
                },
            )
            .await?,
        )
    } else {
        None
    };
    let refund = order_refunds::ActiveModel {
        refund_no: Set(refund_no),
        order_id: Set(order.id),
        amount_cents: Set(order.amount_cents),
        provider: Set(order.provider.clone()),
        provider_trade_no: Set(order.provider_trade_no.clone()),
        refund_reference: Set(refund_reference.clone()),
        reason: Set(reason.clone()),
        status: Set(REFUND_STATUS_SUCCEEDED),
        operator_user_id: Set(operator_user_id),
        refunded_at: Set(now),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&tx)
    .await?;

    if attachment.is_some() {
        order_refund_attachments::ActiveModel {
            refund_id: Set(refund.id),
            resource_id: Set(resource.as_ref().map(|value| value.id).ok_or_else(|| {
                AppError::business_logic(
                    "REFUND_RESOURCE_MISSING",
                    "refund resource metadata is missing",
                )
            })?),
            uploaded_by: Set(operator_user_id),
            created_at: Set(now),
        }
        .insert(&tx)
        .await?;
    }

    let mut active = order.into_active_model();
    active.status = Set(i16::from(OrderStatus::Refunded));
    active.updated_at = Set(now);
    let updated_order = active.update(&tx).await?;

    let event = create_order_event(
        &tx,
        &updated_order,
        ORDER_EVENT_REFUND_CONFIRMED,
        json!({
            "refund_no": refund.refund_no,
            "refund_reference": refund_reference,
            "reason": reason,
            "amount_cents": refund.amount_cents,
            "provider": refund.provider,
            "provider_trade_no": refund.provider_trade_no,
            "operator_user_id": operator_user_id,
            "attachment_file_name": resource.as_ref().map(|value| value.original_name.clone()),
            "attachment_content_type": resource.as_ref().map(|value| value.content_type.clone()),
            "attachment_size": resource.as_ref().map(|value| value.size),
        }),
    )
    .await?;
    notify_order_event(&tx, event.id).await?;
    tx.commit().await?;

    get_order_by_no_impl(state, &updated_order.order_no).await
}

#[handler]
pub async fn refund_attachment(
    depot: &mut Depot,
    id: PathParam<i32>,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let refund = order_refunds::Entity::find()
        .filter(order_refunds::Column::OrderId.eq(id.into_inner()))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("order_refunds", None))?;
    let attachment = order_refund_attachments::Entity::find_by_id(refund.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("order_refund_attachments", None))?;
    let resource = resources::Entity::find_by_id(attachment.resource_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("resources", None))?;
    let downloaded = download_resource(state, &resource).await?;
    let content_type = resource.content_type.clone();
    res.headers_mut().insert(
        salvo::http::header::CONTENT_TYPE,
        salvo::http::HeaderValue::from_str(&content_type)
            .map_err(|_| AppError::validation("invalid refund attachment content type"))?,
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
            message: format!("failed to write refund attachment: {error}"),
        })?;
    Ok(())
}

pub async fn get_order_by_no_impl(state: &AppState, order_no: &str) -> Result<OrderInfo, AppError> {
    let row = orders::Entity::find()
        .filter(orders::Column::OrderNo.eq(order_no))
        .find_also_related(license_plans::Entity)
        .find_also_related(apps::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("orders", None))?;
    build_order_info(state, row).await
}

async fn sync_pending_order_from_provider(
    state: &AppState,
    order: OrderInfo,
) -> Result<OrderInfo, AppError> {
    if order.status != OrderStatus::Pending
        || !state.config.payment.enabled
        || order.provider != PROVIDER_WECHAT
    {
        return Ok(order);
    }

    let channel = match find_payment_channel_by_pay_type(state, &order.pay_type)
        .await?
        .filter(|channel| {
            PaymentChannelStatus::from(channel.status) == PaymentChannelStatus::Enabled
        }) {
        Some(channel) => channel,
        None => return Ok(order),
    };
    let adapter = build_payment_adapter(&channel)?;
    match adapter.query_payment(&order.order_no).await {
        Ok(Some(mut notification)) if notification.status == PaymentStatus::Success => {
            notification.provider = channel.provider.clone();
            notification.pay_type = channel.pay_type.clone();
            match process_payment_notification(state, notification).await {
                Ok(updated) => Ok(updated),
                Err(error) => {
                    tracing::warn!(
                        order_no = %order.order_no,
                        pay_type = %order.pay_type,
                        "payment query succeeded but order sync failed: {}",
                        error
                    );
                    Ok(order)
                }
            }
        }
        Ok(_) => Ok(order),
        Err(error) => {
            tracing::warn!(
                order_no = %order.order_no,
                pay_type = %order.pay_type,
                "failed to query pending payment status: {}",
                error
            );
            Ok(order)
        }
    }
}

async fn build_order_info(
    state: &AppState,
    row: (
        orders::Model,
        Option<license_plans::Model>,
        Option<apps::Model>,
    ),
) -> Result<OrderInfo, AppError> {
    let reg_code_id = row.0.reg_code_id;
    let mut info = OrderInfo::from(row);
    if let Some(id) = reg_code_id {
        info.reg_code = reg_codes::Entity::find_by_id(id)
            .one(&state.db)
            .await?
            .map(|r| r.code);
    }
    if let Some(refund) = order_refunds::Entity::find()
        .filter(order_refunds::Column::OrderId.eq(info.id))
        .one(&state.db)
        .await?
    {
        let resource = if let Some(attachment) =
            order_refund_attachments::Entity::find_by_id(refund.id)
                .one(&state.db)
                .await?
        {
            resources::Entity::find_by_id(attachment.resource_id)
                .one(&state.db)
                .await?
        } else {
            None
        };
        info.refund = Some(OrderRefundInfo {
            refund_no: refund.refund_no,
            refund_reference: refund.refund_reference,
            reason: refund.reason,
            operator_user_id: refund.operator_user_id,
            attachment_file_name: resource.as_ref().map(|value| value.original_name.clone()),
            attachment_content_type: resource.as_ref().map(|value| value.content_type.clone()),
            attachment_size: resource.as_ref().map(|value| value.size),
            refunded_at: refund.refunded_at,
        });
    }
    Ok(info)
}

#[handler]
pub async fn wechat_native_notify(depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let result = handle_payment_notify(state, PAY_TYPE_WECHAT_NATIVE, req).await;
    match result {
        Ok(_) => {
            res.status_code(StatusCode::OK);
            res.render(Json(json!({
                "code": "SUCCESS",
                "message": "成功"
            })));
        }
        Err(error) => {
            tracing::error!("WeChat Native payment notification failed: {}", error);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "code": "FAIL",
                "message": error.to_string()
            })));
        }
    }
}

#[handler]
pub async fn alipay_notify(depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let result = handle_payment_notify(state, PAY_TYPE_ALIPAY, req).await;
    match result {
        Ok(_) => {
            res.status_code(StatusCode::OK);
            res.render("success");
        }
        Err(error) => {
            tracing::error!("Alipay payment notification failed: {}", error);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render("failure");
        }
    }
}

#[handler]
pub async fn payment_notify(
    depot: &mut Depot,
    pay_type: PathParam<String>,
    req: &mut Request,
    res: &mut Response,
) {
    let state = depot.obtain::<AppState>().unwrap();
    let pay_type = pay_type.into_inner();
    let provider = payment_channel_provider(state, &pay_type)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            provider_for_pay_type(&pay_type)
                .unwrap_or(PROVIDER_WECHAT)
                .to_string()
        });
    let result = handle_payment_notify(state, &pay_type, req).await;
    match (provider.as_str(), result) {
        (PROVIDER_ALIPAY, Ok(_)) => {
            res.status_code(StatusCode::OK);
            res.render("success");
        }
        (PROVIDER_ALIPAY, Err(error)) => {
            tracing::error!("Payment notification failed: {}", error);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render("failure");
        }
        (_, Ok(_)) => {
            res.status_code(StatusCode::OK);
            res.render(Json(json!({
                "code": "SUCCESS",
                "message": "success"
            })));
        }
        (_, Err(error)) => {
            tracing::error!("Payment notification failed: {}", error);
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(json!({
                "code": "FAIL",
                "message": error.to_string()
            })));
        }
    }
}

async fn handle_payment_notify(
    state: &AppState,
    pay_type: &str,
    req: &mut Request,
) -> Result<OrderInfo, AppError> {
    let pay_type = normalize_pay_type(pay_type)?;
    let channel = find_payment_channel_by_pay_type(state, &pay_type)
        .await?
        .filter(|channel| {
            PaymentChannelStatus::from(channel.status) == PaymentChannelStatus::Enabled
        })
        .ok_or_else(|| {
            AppError::business_logic(
                "PAYMENT_CHANNEL_UNAVAILABLE",
                "payment channel is not configured or disabled",
            )
        })?;
    let adapter = build_payment_adapter(&channel)?;
    let headers = collect_payment_headers(req);
    let body = req
        .payload_with_max_size(1024 * 1024)
        .await
        .map_err(|error| AppError::Message(format!("failed to read payment notify: {}", error)))?;
    let mut notification = adapter
        .parse_notification(&headers, body.as_ref())
        .await
        .map_err(payment_error)?;
    notification.provider = channel.provider.clone();
    notification.pay_type = channel.pay_type.clone();
    process_payment_notification(state, notification).await
}

pub async fn process_payment_notification(
    state: &AppState,
    notification: PaymentNotification,
) -> Result<OrderInfo, AppError> {
    if notification.status != PaymentStatus::Success {
        return Err(AppError::business_logic(
            "PAYMENT_NOT_SUCCESS",
            "payment notification is not success",
        ));
    }

    let tx = state.db.begin().await?;
    let mut order = orders::Entity::find()
        .filter(orders::Column::OrderNo.eq(notification.out_trade_no.clone()))
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("orders", None))?;

    if notification.provider != order.provider || notification.pay_type != order.pay_type {
        return Err(AppError::business_logic(
            "PAYMENT_PROVIDER_MISMATCH",
            "payment notification provider does not match order",
        ));
    }

    if OrderStatus::from(order.status) == OrderStatus::Refunded {
        tx.commit().await?;
        return get_order_by_no_impl(state, &notification.out_trade_no).await;
    }

    if OrderStatus::from(order.status) == OrderStatus::Delivered {
        if let Some(reg_code_id) = order.reg_code_id {
            let now = Utc::now().fixed_offset();
            reg_codes::Entity::update_many()
                .col_expr(
                    reg_codes::Column::Status,
                    sea_orm::sea_query::Expr::value(i16::from(RegCodeStatus::Issued)),
                )
                .col_expr(
                    reg_codes::Column::UpdatedAt,
                    sea_orm::sea_query::Expr::value(now),
                )
                .filter(reg_codes::Column::Id.eq(reg_code_id))
                .filter(reg_codes::Column::Status.eq(i16::from(RegCodeStatus::Unused)))
                .exec(&tx)
                .await?;
        }
        tx.commit().await?;
        return get_order_by_no_impl(state, &notification.out_trade_no).await;
    }

    if notification.amount_cents != order.amount_cents {
        return Err(AppError::business_logic(
            "AMOUNT_MISMATCH",
            "payment amount does not match order amount",
        ));
    }

    if order.buyer_user_id.is_none() {
        if let Some(email) = order.buyer_email.as_deref() {
            if let Some(user) = users::Entity::find()
                .filter(users::Column::Email.eq(email))
                .filter(users::Column::EmailVerifiedAt.is_not_null())
                .one(&tx)
                .await?
            {
                order.buyer_user_id = Some(user.id);
                if order.referrer_user_id == Some(user.id) {
                    order.referrer_user_id = None;
                    order.referral_code = None;
                    order.commission_rate_bps = None;
                    order.commission_amount_cents = None;
                }
            }
        }
    }

    let plan = license_plans::Entity::find_by_id(order.plan_id)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(order.plan_id)))?;
    let reg_code = create_paid_reg_code(&tx, &plan).await?;

    let now = Utc::now().fixed_offset();
    let mut active = order.into_active_model();
    active.status = Set(i16::from(OrderStatus::Delivered));
    active.provider_trade_no = Set(notification.provider_trade_no.clone());
    active.reg_code_id = Set(Some(reg_code.id));
    active.provider_payload = Set(Some(notification.raw_payload.clone()));
    active.paid_at = Set(Some(now));
    active.updated_at = Set(now);
    let updated_order = active.update(&tx).await?;
    if let (Some(user_id), Some(rate), Some(amount)) = (
        updated_order.referrer_user_id,
        updated_order.commission_rate_bps,
        updated_order.commission_amount_cents,
    ) {
        let distribution = get_distribution_settings(state).await?;
        let available_at = now + chrono::Duration::days(distribution.holding_days as i64);
        new_commission_active_model(
            updated_order.id,
            user_id,
            updated_order.amount_cents,
            rate,
            amount,
            available_at,
            now,
        )
        .insert(&tx)
        .await?;
    }
    let event = create_order_event(
        &tx,
        &updated_order,
        ORDER_EVENT_PAYMENT_DELIVERED,
        json!({
            "provider": notification.provider,
            "pay_type": notification.pay_type,
            "provider_trade_no": notification.provider_trade_no,
            "reg_code_id": reg_code.id,
        }),
    )
    .await?;
    notify_order_event(&tx, event.id).await?;
    tx.commit().await?;

    get_order_by_no_impl(state, &notification.out_trade_no).await
}

async fn create_order_event(
    tx: &DatabaseTransaction,
    order: &orders::Model,
    event_type: &str,
    payload: serde_json::Value,
) -> Result<order_events::Model, AppError> {
    let now = Utc::now().fixed_offset();
    let active = order_events::ActiveModel {
        order_id: Set(order.id),
        order_no: Set(order.order_no.clone()),
        event_type: Set(event_type.to_string()),
        status: Set(0),
        payload: Set(Some(payload)),
        created_at: Set(now),
        ..Default::default()
    };
    Ok(active.insert(tx).await?)
}

async fn notify_order_event(tx: &DatabaseTransaction, event_id: i64) -> Result<(), AppError> {
    let sql = "SELECT pg_notify($1, $2)";
    tx.execute(Statement::from_sql_and_values(
        tx.get_database_backend(),
        sql,
        vec![
            ORDER_EVENTS_NOTIFY_CHANNEL.into(),
            json!({ "event_id": event_id }).to_string().into(),
        ],
    ))
    .await?;
    Ok(())
}

async fn create_paid_reg_code(
    tx: &DatabaseTransaction,
    plan: &license_plans::Model,
) -> Result<reg_codes::Model, AppError> {
    let app = apps::Entity::find_by_id(plan.app_id)
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("apps", Some(plan.app_id)))?;
    let now = Utc::now().fixed_offset();
    let active = reg_codes::ActiveModel {
        code: Set(new_reg_code()),
        app_id: Set(plan.app_id),
        valid_days: Set(plan.valid_days),
        max_devices: Set(app.max_devices),
        status: Set(i16::from(RegCodeStatus::Issued)),
        code_type: Set(plan.code_type),
        total_count: Set(plan.total_count),
        remaining_count: Set((app.max_devices > 1).then_some(plan.total_count).flatten()),
        multi_device_enabled: Set(app.max_devices > 1),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    Ok(active.insert(tx).await?)
}

fn validate_plan_input(
    price_cents: i32,
    code_type: CodeType,
    valid_days: i32,
    total_count: Option<i32>,
) -> Result<(), AppError> {
    if price_cents <= 0 {
        return Err(AppError::validation("price_cents must be greater than 0"));
    }
    match code_type {
        CodeType::Time if valid_days <= 0 => {
            Err(AppError::validation("valid_days must be greater than 0"))
        }
        CodeType::Count if total_count.unwrap_or(0) <= 0 => {
            Err(AppError::validation("total_count must be greater than 0"))
        }
        _ => Ok(()),
    }
}

async fn ensure_plan_matches_app(
    state: &AppState,
    app_id: i32,
    code_type: CodeType,
) -> Result<(), AppError> {
    let app = apps::Entity::find_by_id(app_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("apps", Some(app_id)))?;
    if CodeType::from(app.code_type) != code_type {
        return Err(AppError::business_logic(
            "PLAN_APP_CODE_TYPE_MISMATCH",
            "plan code_type must match app code_type",
        ));
    }
    Ok(())
}

fn collect_payment_headers(req: &Request) -> PaymentHeaders {
    let mut headers = PaymentHeaders::new();
    for (name, value) in req.headers() {
        if let Ok(value) = value.to_str() {
            headers.insert(name.as_str().to_ascii_lowercase(), value.to_string());
        }
    }
    headers
}

fn payment_error(error: PaymentError) -> AppError {
    AppError::ExternalService {
        service: "payment".to_string(),
        error: error.to_string(),
    }
}

async fn find_payment_channel_by_pay_type(
    state: &AppState,
    pay_type: &str,
) -> Result<Option<payment_channels::Model>, AppError> {
    Ok(payment_channels::Entity::find()
        .filter(payment_channels::Column::PayType.eq(pay_type))
        .one(&state.db)
        .await?)
}

async fn payment_channel_provider(
    state: &AppState,
    pay_type: &str,
) -> Result<Option<String>, AppError> {
    let pay_type = normalize_pay_type(pay_type)?;
    Ok(find_payment_channel_by_pay_type(state, &pay_type)
        .await?
        .map(|channel| channel.provider))
}

fn build_payment_adapter(
    channel: &payment_channels::Model,
) -> Result<Box<dyn PaymentAdapter>, AppError> {
    match channel.provider.as_str() {
        PROVIDER_WECHAT => {
            let config = parse_wechat_config(&channel.config)?;
            Ok(Box::new(
                WechatNativeAdapter::new(WechatNativeConfig {
                    app_id: config.app_id,
                    mch_id: config.mch_id,
                    merchant_serial_no: config.merchant_serial_no,
                    merchant_private_key_pem: config.merchant_private_key_pem,
                    api_v3_key: config.api_v3_key,
                    wechatpay_public_key_id: config.wechatpay_public_key_id,
                    wechatpay_public_key_pem: config.wechatpay_public_key_pem,
                    api_base_url: config.api_base_url,
                })
                .map_err(payment_error)?,
            ))
        }
        PROVIDER_ALIPAY => {
            let config = parse_alipay_config(&channel.config)?;
            Ok(Box::new(
                AlipayPageAdapter::new(AlipayPageConfig {
                    app_id: config.app_id,
                    app_private_key_pem: config.app_private_key_pem,
                    alipay_public_key_pem: config.alipay_public_key_pem,
                    gateway_url: config.gateway_url,
                    seller_id: non_empty_string(config.seller_id),
                })
                .map_err(payment_error)?,
            ))
        }
        _ => Err(AppError::validation("payment provider is not supported")),
    }
}

fn normalize_channel_config(provider: &str, config: &Value) -> Result<Value, AppError> {
    match provider {
        PROVIDER_WECHAT => serde_json::to_value(parse_wechat_config(config)?)
            .map_err(|error| AppError::validation(format!("invalid WeChat config: {}", error))),
        PROVIDER_ALIPAY => serde_json::to_value(parse_alipay_config(config)?)
            .map_err(|error| AppError::validation(format!("invalid Alipay config: {}", error))),
        _ => Err(AppError::validation("payment provider is not supported")),
    }
}

fn response_channel_config(provider: &str, config: &Value) -> Result<Value, AppError> {
    match provider {
        PROVIDER_WECHAT => Ok(json!({
            "app_id": config_text(config, "app_id"),
            "mch_id": config_text(config, "mch_id"),
            "merchant_serial_no": config_text(config, "merchant_serial_no"),
            "merchant_private_key_pem": config_text(config, "merchant_private_key_pem"),
            "api_v3_key": config_text(config, "api_v3_key"),
            "wechatpay_public_key_id": config_text(config, "wechatpay_public_key_id"),
            "wechatpay_public_key_pem": config_text(config, "wechatpay_public_key_pem"),
            "api_base_url": non_empty_string(config_text(config, "api_base_url"))
                .unwrap_or_else(|| DEFAULT_WECHAT_API_BASE_URL.to_string()),
        })),
        PROVIDER_ALIPAY => {
            let normalized = parse_alipay_config(config)?;
            serde_json::to_value(normalized)
                .map_err(|error| AppError::validation(format!("invalid Alipay config: {}", error)))
        }
        _ => Err(AppError::validation("payment provider is not supported")),
    }
}

fn parse_wechat_config(config: &Value) -> Result<WechatChannelConfig, AppError> {
    ensure_config_object(config)?;
    let api_base_url = non_empty_string(config_text(config, "api_base_url"))
        .unwrap_or_else(|| DEFAULT_WECHAT_API_BASE_URL.to_string());
    Ok(WechatChannelConfig {
        app_id: required_config_text(config, "app_id")?,
        mch_id: required_config_text(config, "mch_id")?,
        merchant_serial_no: required_config_text(config, "merchant_serial_no")?,
        merchant_private_key_pem: required_config_text(config, "merchant_private_key_pem")?,
        api_v3_key: required_config_text(config, "api_v3_key")?,
        wechatpay_public_key_id: required_config_text(config, "wechatpay_public_key_id")?,
        wechatpay_public_key_pem: required_config_text(config, "wechatpay_public_key_pem")?,
        api_base_url,
    })
}

fn parse_alipay_config(config: &Value) -> Result<AlipayChannelConfig, AppError> {
    ensure_config_object(config)?;
    let gateway_url = non_empty_string(config_text(config, "gateway_url"))
        .unwrap_or_else(|| DEFAULT_ALIPAY_GATEWAY_URL.to_string());
    Ok(AlipayChannelConfig {
        app_id: required_config_text(config, "app_id")?,
        app_private_key_pem: required_config_text(config, "app_private_key_pem")?,
        alipay_public_key_pem: required_config_text(config, "alipay_public_key_pem")?,
        gateway_url,
        seller_id: config_text(config, "seller_id"),
    })
}

fn ensure_config_object(config: &Value) -> Result<(), AppError> {
    if config.is_object() {
        Ok(())
    } else {
        Err(AppError::validation(
            "payment channel config must be an object",
        ))
    }
}

fn required_config_text(config: &Value, key: &str) -> Result<String, AppError> {
    let value = config_text(config, key);
    if value.is_empty() {
        return Err(AppError::validation(format!(
            "payment channel config field '{}' is required",
            key
        )));
    }
    Ok(value)
}

fn config_text(config: &Value, key: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn normalize_required_text(value: String, field: &str) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(AppError::validation(format!("{} is required", field)));
    }
    Ok(value)
}

fn normalize_provider(provider: &str) -> Result<String, AppError> {
    let provider = provider.trim().to_ascii_lowercase();
    match provider.as_str() {
        PROVIDER_WECHAT | PROVIDER_ALIPAY => Ok(provider),
        _ => Err(AppError::validation("payment provider is not supported")),
    }
}

fn validate_channel_status(status: i16) -> Result<(), AppError> {
    match status {
        0 | 1 => Ok(()),
        _ => Err(AppError::validation(
            "payment channel status is not supported",
        )),
    }
}

fn notify_url_for_pay_type(state: &AppState, pay_type: &str) -> Result<String, AppError> {
    let pay_type = normalize_pay_type(pay_type)?;
    Ok(format!(
        "{}/api/pay/{}/notify",
        state.config.payment.public_base_url.trim_end_matches('/'),
        pay_type
    ))
}

fn return_url_for_provider(state: &AppState, provider: &str, order_no: &str) -> Option<String> {
    match provider {
        PROVIDER_ALIPAY => Some(format!(
            "{}/pay/result?order_no={}&result=pending",
            state.config.payment.frontend_base_url.trim_end_matches('/'),
            order_no
        )),
        _ => None,
    }
}

fn provider_for_pay_type(pay_type: &str) -> Result<&'static str, AppError> {
    match pay_type {
        PAY_TYPE_WECHAT_NATIVE => Ok(PROVIDER_WECHAT),
        PAY_TYPE_ALIPAY | "alipay_page" => Ok(PROVIDER_ALIPAY),
        _ => Err(AppError::validation("pay_type is not supported")),
    }
}

fn normalize_pay_type(pay_type: &str) -> Result<String, AppError> {
    let pay_type = pay_type.trim().to_string();
    if pay_type.is_empty() {
        return Err(AppError::validation("pay_type is required"));
    }
    if pay_type.len() > 64 {
        return Err(AppError::validation("pay_type must be at most 64 bytes"));
    }
    if !pay_type
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(AppError::validation(
            "pay_type may only contain letters, numbers, '_' and '-'",
        ));
    }
    Ok(pay_type)
}

fn non_empty_string(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn new_order_no() -> String {
    format!("LH{}{}", Utc::now().format("%Y%m%d%H%M%S"), short_id(10))
}

fn new_reg_code() -> String {
    format!("LH-{}", short_id(20))
}

fn short_id(len: usize) -> String {
    Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(len)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}
