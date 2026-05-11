use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::apis::reg_codes_handler::{CodeType, RegCodeStatus};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::Utc;
use data_model::{apps, license_plans, order_events, orders, reg_codes};
use salvo::http::Method;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use uuid::Uuid;
use validator::Validate;

const PROVIDER_CAIDOU: &str = "caidou";
const ORDER_EVENT_PAYMENT_DELIVERED: &str = "payment.delivered";
const ORDER_EVENTS_NOTIFY_CHANNEL: &str = "licensehub_order_events";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i16)]
#[serde(from = "i16", into = "i16")]
pub enum PlanStatus {
    Disabled = 0,
    Enabled = 1,
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
}

impl From<i16> for OrderStatus {
    fn from(value: i16) -> Self {
        match value {
            1 => OrderStatus::Paid,
            2 => OrderStatus::Delivered,
            3 => OrderStatus::Failed,
            4 => OrderStatus::Closed,
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

#[derive(Debug, Serialize)]
pub struct PlanInfo {
    pub id: i32,
    pub app_id: i32,
    pub app_name: Option<String>,
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
        Self {
            id: plan.id,
            app_id: plan.app_id,
            app_name: app.map(|a| a.name),
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

#[derive(Debug, Deserialize)]
pub struct CreateOrderReq {
    pub plan_id: i32,
    pub pay_type: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ListOrdersParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub order_no: Option<String>,
    pub status: Option<i16>,
    pub plan_id: Option<i32>,
    pub app_id: Option<i32>,
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
            paid_at: order.paid_at,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CaidouNotify {
    pub pid: String,
    pub trade_no: String,
    pub out_trade_no: String,
    #[serde(rename = "type")]
    pub pay_type: String,
    pub name: String,
    pub money: String,
    pub trade_status: String,
    pub param: Option<String>,
    pub sign: String,
    pub sign_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CaidouMapiResp {
    code: i32,
    msg: Option<String>,
    trade_no: Option<String>,
    payurl: Option<String>,
    qrcode: Option<String>,
    urlscheme: Option<String>,
    money: Option<String>,
}

struct CaidouPaymentRequest {
    params: BTreeMap<String, String>,
    submit_url: String,
}

#[derive(Debug, Serialize)]
pub struct PayMethodInfo {
    pub pay_type: String,
    pub label: String,
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
) -> Result<ApiResponse<Vec<PlanInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<PublicPlansParams>().unwrap_or_default();
    let mut query = license_plans::Entity::find()
        .find_also_related(apps::Entity)
        .filter(license_plans::Column::Status.eq(i16::from(PlanStatus::Enabled)))
        .order_by_asc(license_plans::Column::SortOrder)
        .order_by_asc(license_plans::Column::Id);
    if let Some(app_id) = params.app_id {
        query = query.filter(license_plans::Column::AppId.eq(app_id));
    }
    let rows = query.all(&state.db).await?;
    Ok(ApiResponse::success(
        rows.into_iter().map(PlanInfo::from).collect(),
    ))
}

#[handler]
pub async fn list_pay_methods(depot: &mut Depot) -> Result<ApiResponse<PayMethodsInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    Ok(ApiResponse::success(fetch_pay_methods_impl(state).await))
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

async fn fetch_pay_methods_impl(state: &AppState) -> PayMethodsInfo {
    let cfg = &state.config.caidou_pay;
    if !cfg.enabled {
        return PayMethodsInfo {
            enabled: false,
            provider: PROVIDER_CAIDOU.to_string(),
            merchant_active: false,
            methods: Vec::new(),
            message: Some("payment is disabled".to_string()),
        };
    }
    if cfg.pid.is_empty() || cfg.key.is_empty() {
        return PayMethodsInfo {
            enabled: false,
            provider: PROVIDER_CAIDOU.to_string(),
            merchant_active: false,
            methods: Vec::new(),
            message: Some("CAIDOU_PID and CAIDOU_KEY must be set".to_string()),
        };
    }

    match request_caidou_merchant_info(state).await {
        Ok(info) => {
            let merchant_active = json_value_is_active(info.get("active"));
            let methods = if merchant_active {
                resolve_pay_methods_from_merchant_info(&info, &cfg.pay_types)
            } else {
                Vec::new()
            };
            PayMethodsInfo {
                enabled: merchant_active && !methods.is_empty(),
                provider: PROVIDER_CAIDOU.to_string(),
                merchant_active,
                methods,
                message: if merchant_active {
                    None
                } else {
                    Some("merchant is inactive".to_string())
                },
            }
        }
        Err(error) => PayMethodsInfo {
            enabled: false,
            provider: PROVIDER_CAIDOU.to_string(),
            merchant_active: false,
            methods: Vec::new(),
            message: Some(error.to_string()),
        },
    }
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
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);
    let mut query = license_plans::Entity::find()
        .find_also_related(apps::Entity)
        .order_by_desc(license_plans::Column::CreatedAt);
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
    let client_ip = req.remote_addr().to_string();
    let order = create_order_impl(state, body, Some(client_ip)).await?;
    Ok(ApiResponse::success(order))
}

pub async fn create_order_impl(
    state: &AppState,
    req: CreateOrderReq,
    client_ip: Option<String>,
) -> Result<OrderInfo, AppError> {
    let pay_type = normalize_pay_type(&req.pay_type)?;
    let plan = license_plans::Entity::find_by_id(req.plan_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(req.plan_id)))?;
    if PlanStatus::from(plan.status) != PlanStatus::Enabled {
        return Err(AppError::business_logic(
            "PLAN_DISABLED",
            "plan is disabled",
        ));
    }
    if plan.price_cents <= 0 {
        return Err(AppError::business_logic(
            "INVALID_PRICE",
            "plan price is invalid",
        ));
    }

    let now = Utc::now().fixed_offset();
    let order_no = new_order_no();
    let mut active = orders::ActiveModel {
        order_no: Set(order_no.clone()),
        plan_id: Set(plan.id),
        app_id: Set(plan.app_id),
        amount_cents: Set(plan.price_cents),
        pay_type: Set(pay_type.clone()),
        status: Set(i16::from(OrderStatus::Pending)),
        provider: Set(PROVIDER_CAIDOU.to_string()),
        client_ip: Set(client_ip),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    if state.config.caidou_pay.enabled {
        let pay_req = build_caidou_payment_request(state, &plan, &order_no, &pay_type)?;
        let pay_resp = request_caidou_payment(state, &pay_req.params).await?;
        active.provider_trade_no = Set(pay_resp.trade_no.clone());
        active.pay_url = Set(Some(pay_req.submit_url));
        active.qr_code = Set(pay_resp.qrcode.clone());
        active.url_scheme = Set(pay_resp.urlscheme.clone());
        active.provider_payload = Set(Some(json!(&pay_resp)));
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
    let order = get_order_by_no_impl(state, &order_no.into_inner()).await?;
    Ok(ApiResponse::success(PublicOrderInfo::from(order)))
}

#[handler]
pub async fn list_orders(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<OrderInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<ListOrdersParams>()?;
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);
    let mut query = orders::Entity::find()
        .find_also_related(license_plans::Entity)
        .find_also_related(apps::Entity)
        .order_by_desc(orders::Column::CreatedAt);
    if let Some(order_no) = params.order_no {
        query = query.filter(orders::Column::OrderNo.contains(order_no));
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
    Ok(info)
}

#[handler]
pub async fn caidou_notify(
    depot: &mut Depot,
    req: &mut Request,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let notify = parse_notify(req).await?;
    process_caidou_notify(state, notify).await?;
    res.render(Text::Plain("success"));
    Ok(())
}

#[handler]
pub async fn caidou_return(
    depot: &mut Depot,
    req: &mut Request,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let notify = parse_notify(req).await?;
    let order_no = notify.out_trade_no.clone();
    let result = match process_caidou_notify(state, notify).await {
        Ok(_) => "success",
        Err(error) => {
            tracing::error!("Caidou return processing failed: {}", error);
            "failed"
        }
    };
    let redirect_url = format!(
        "{}/pay/result?order_no={}&result={}",
        state.config.caidou_pay.frontend_base_url,
        url_encode(&order_no),
        result
    );
    res.render(Redirect::other(redirect_url));
    Ok(())
}

pub async fn process_caidou_notify(
    state: &AppState,
    notify: CaidouNotify,
) -> Result<OrderInfo, AppError> {
    verify_notify(state, &notify)?;
    if notify.trade_status != "TRADE_SUCCESS" {
        return Err(AppError::business_logic(
            "PAYMENT_NOT_SUCCESS",
            "trade_status is not TRADE_SUCCESS",
        ));
    }

    let tx = state.db.begin().await?;
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNo.eq(notify.out_trade_no.clone()))
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("orders", None))?;

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
        return get_order_by_no_impl(state, &notify.out_trade_no).await;
    }

    let amount_cents = money_to_cents(&notify.money)?;
    if amount_cents != order.amount_cents {
        return Err(AppError::business_logic(
            "AMOUNT_MISMATCH",
            "payment amount does not match order amount",
        ));
    }

    let plan = license_plans::Entity::find_by_id(order.plan_id)
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::not_found("license_plans", Some(order.plan_id)))?;
    let reg_code = create_paid_reg_code(&tx, &plan).await?;

    let now = Utc::now().fixed_offset();
    let mut active = order.into_active_model();
    active.status = Set(i16::from(OrderStatus::Delivered));
    active.provider_trade_no = Set(Some(notify.trade_no.clone()));
    active.reg_code_id = Set(Some(reg_code.id));
    active.provider_payload = Set(Some(json!(notify)));
    active.paid_at = Set(Some(now));
    active.updated_at = Set(now);
    let updated_order = active.update(&tx).await?;
    let event = create_order_event(
        &tx,
        &updated_order,
        ORDER_EVENT_PAYMENT_DELIVERED,
        json!({
            "provider": PROVIDER_CAIDOU,
            "provider_trade_no": notify.trade_no,
            "reg_code_id": reg_code.id,
        }),
    )
    .await?;
    notify_order_event(&tx, event.id).await?;
    tx.commit().await?;

    get_order_by_no_impl(state, &notify.out_trade_no).await
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

async fn parse_notify(req: &mut Request) -> Result<CaidouNotify, AppError> {
    if req.method() == Method::GET {
        return Ok(req.parse_queries::<CaidouNotify>()?);
    }
    Ok(req.parse_body::<CaidouNotify>().await?)
}

fn verify_notify(state: &AppState, notify: &CaidouNotify) -> Result<(), AppError> {
    let mut params = BTreeMap::new();
    params.insert("money".to_string(), notify.money.clone());
    params.insert("name".to_string(), notify.name.clone());
    if let Some(param) = &notify.param {
        params.insert("param".to_string(), param.clone());
    }
    params.insert("out_trade_no".to_string(), notify.out_trade_no.clone());
    params.insert("pid".to_string(), notify.pid.clone());
    params.insert("trade_no".to_string(), notify.trade_no.clone());
    params.insert("trade_status".to_string(), notify.trade_status.clone());
    params.insert("type".to_string(), notify.pay_type.clone());
    let expected = caidou_sign(&params, &state.config.caidou_pay.key);
    if expected != notify.sign.to_ascii_lowercase() {
        return Err(AppError::business_logic(
            "INVALID_SIGN",
            "payment notification signature is invalid",
        ));
    }
    if notify.pid != state.config.caidou_pay.pid {
        return Err(AppError::business_logic(
            "INVALID_PID",
            "payment notification merchant id is invalid",
        ));
    }
    Ok(())
}

fn build_caidou_payment_request(
    state: &AppState,
    plan: &license_plans::Model,
    order_no: &str,
    pay_type: &str,
) -> Result<CaidouPaymentRequest, AppError> {
    let cfg = &state.config.caidou_pay;
    if cfg.pid.is_empty() || cfg.key.is_empty() {
        return Err(AppError::validation(
            "CAIDOU_PID and CAIDOU_KEY must be set",
        ));
    }
    let mut params = BTreeMap::new();
    params.insert("pid".to_string(), cfg.pid.clone());
    params.insert("type".to_string(), pay_type.to_string());
    params.insert("out_trade_no".to_string(), order_no.to_string());
    params.insert(
        "notify_url".to_string(),
        format!("{}/api/pay/caidou/notify", cfg.public_base_url),
    );
    params.insert(
        "return_url".to_string(),
        format!("{}/api/pay/caidou/return", cfg.public_base_url),
    );
    params.insert("name".to_string(), plan.name.clone());
    params.insert("money".to_string(), cents_to_money(plan.price_cents));
    params.insert("sitename".to_string(), cfg.site_name.clone());
    params.insert("param".to_string(), plan.id.to_string());
    params.insert("device".to_string(), "pc".to_string());
    let sign = caidou_sign(&params, &cfg.key);
    params.insert("sign".to_string(), sign);
    params.insert("sign_type".to_string(), "MD5".to_string());

    Ok(CaidouPaymentRequest {
        submit_url: caidou_submit_url(cfg.base_url.as_str(), &params),
        params,
    })
}

async fn request_caidou_payment(
    state: &AppState,
    params: &BTreeMap<String, String>,
) -> Result<CaidouMapiResp, AppError> {
    let cfg = &state.config.caidou_pay;
    let url = format!("{}/xpay/epay/mapi.php", cfg.base_url);
    let resp = reqwest::Client::new()
        .post(&url)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::ExternalService {
            service: "caidou".to_string(),
            error: e.to_string(),
        })?;
    let status = resp.status();
    let parsed = resp
        .json::<CaidouMapiResp>()
        .await
        .map_err(|e| AppError::ExternalService {
            service: "caidou".to_string(),
            error: format!("invalid response {}: {}", status, e),
        })?;
    if parsed.code != 1 {
        return Err(AppError::ExternalService {
            service: "caidou".to_string(),
            error: parsed
                .msg
                .clone()
                .unwrap_or_else(|| "payment request failed".to_string()),
        });
    }
    Ok(parsed)
}

async fn request_caidou_merchant_info(
    state: &AppState,
) -> Result<serde_json::Map<String, serde_json::Value>, AppError> {
    let cfg = &state.config.caidou_pay;
    let url = format!("{}/xpay/epay/api.php", cfg.base_url);
    let resp = reqwest::Client::new()
        .get(&url)
        .query(&[
            ("act", "query"),
            ("pid", cfg.pid.as_str()),
            ("key", cfg.key.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::ExternalService {
            service: "caidou".to_string(),
            error: e.to_string(),
        })?;
    let status = resp.status();
    let parsed = resp
        .json::<serde_json::Value>()
        .await
        .map_err(|e| AppError::ExternalService {
            service: "caidou".to_string(),
            error: format!("invalid merchant info response {}: {}", status, e),
        })?;
    parsed
        .as_object()
        .cloned()
        .ok_or_else(|| AppError::ExternalService {
            service: "caidou".to_string(),
            error: "merchant info response is not an object".to_string(),
        })
}

fn resolve_pay_methods_from_merchant_info(
    info: &serde_json::Map<String, serde_json::Value>,
    configured_pay_types: &[String],
) -> Vec<PayMethodInfo> {
    let mut methods = Vec::new();
    for (pay_type, label) in [
        ("alipay", "支付宝"),
        ("wxpay", "微信"),
        ("qqpay", "QQ钱包"),
        ("bank", "网银"),
        ("jdpay", "京东支付"),
    ] {
        if !configured_pay_types.is_empty()
            && !configured_pay_types.iter().any(|item| item == pay_type)
        {
            continue;
        }
        if configured_pay_types.iter().any(|item| item == pay_type)
            || merchant_info_supports_pay_type(info, pay_type)
        {
            methods.push(PayMethodInfo {
                pay_type: pay_type.to_string(),
                label: label.to_string(),
                enabled: true,
            });
        }
    }
    if methods.is_empty() && configured_pay_types.is_empty() {
        methods.push(PayMethodInfo {
            pay_type: "alipay".to_string(),
            label: "支付宝".to_string(),
            enabled: true,
        });
        methods.push(PayMethodInfo {
            pay_type: "wxpay".to_string(),
            label: "微信".to_string(),
            enabled: true,
        });
    }
    methods
}

fn merchant_info_supports_pay_type(
    info: &serde_json::Map<String, serde_json::Value>,
    pay_type: &str,
) -> bool {
    let aliases = match pay_type {
        "alipay" => ["alipay", "ali", "alipay_open", "alipay_status"],
        "wxpay" => ["wxpay", "wechat", "weixin", "wxpay_status"],
        "qqpay" => ["qqpay", "qq", "qqpay_status", "tenpay"],
        "bank" => ["bank", "bankpay", "unionpay", "bank_status"],
        "jdpay" => ["jdpay", "jd", "jdpay_status", "jingdong"],
        _ => [pay_type, pay_type, pay_type, pay_type],
    };
    aliases
        .iter()
        .filter_map(|key| info.get(*key))
        .any(|value| json_value_is_active(Some(value)))
        || info
            .get("paytype")
            .or_else(|| info.get("pay_type"))
            .or_else(|| info.get("pay_types"))
            .or_else(|| info.get("channels"))
            .is_some_and(|value| json_value_contains_pay_type(value, pay_type))
}

fn json_value_contains_pay_type(value: &serde_json::Value, pay_type: &str) -> bool {
    match value {
        serde_json::Value::String(text) => text
            .split(|c: char| c == ',' || c == '|' || c == ';' || c.is_whitespace())
            .any(|part| part.eq_ignore_ascii_case(pay_type)),
        serde_json::Value::Array(items) => items
            .iter()
            .any(|item| json_value_contains_pay_type(item, pay_type)),
        serde_json::Value::Object(map) => map
            .get(pay_type)
            .is_some_and(|enabled| json_value_is_active(Some(enabled))),
        _ => false,
    }
}

fn json_value_is_active(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(v)) => *v,
        Some(serde_json::Value::Number(v)) => v.as_i64().unwrap_or_default() > 0,
        Some(serde_json::Value::String(v)) => matches!(
            v.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on" | "enable" | "enabled" | "active" | "normal"
        ),
        Some(serde_json::Value::Null) | None => true,
        _ => false,
    }
}

async fn create_paid_reg_code(
    tx: &DatabaseTransaction,
    plan: &license_plans::Model,
) -> Result<reg_codes::Model, AppError> {
    let now = Utc::now().fixed_offset();
    let active = reg_codes::ActiveModel {
        code: Set(new_reg_code()),
        app_id: Set(plan.app_id),
        valid_days: Set(plan.valid_days),
        max_devices: Set(1),
        status: Set(i16::from(RegCodeStatus::Issued)),
        code_type: Set(plan.code_type),
        total_count: Set(plan.total_count),
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

pub fn caidou_sign(params: &BTreeMap<String, String>, key: &str) -> String {
    let pairs: Vec<String> = params
        .iter()
        .filter(|(k, v)| !v.is_empty() && k.as_str() != "sign" && k.as_str() != "sign_type")
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    format!("{:x}", md5::compute(format!("{}{}", pairs.join("&"), key)))
}

fn caidou_submit_url(base_url: &str, params: &BTreeMap<String, String>) -> String {
    let query = params
        .iter()
        .map(|(key, value)| format!("{}={}", url_encode(key), url_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    format!("{}/xpay/epay/submit.php?{}", base_url, query)
}

fn url_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

fn normalize_pay_type(pay_type: &str) -> Result<String, AppError> {
    match pay_type {
        "alipay" | "wxpay" | "qqpay" | "bank" | "jdpay" => Ok(pay_type.to_string()),
        _ => Err(AppError::validation("pay_type is not supported")),
    }
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

fn cents_to_money(cents: i32) -> String {
    format!("{}.{:02}", cents / 100, (cents % 100).abs())
}

fn money_to_cents(value: &str) -> Result<i32, AppError> {
    let mut parts = value.split('.');
    let yuan = parts
        .next()
        .unwrap_or("0")
        .parse::<i32>()
        .map_err(|_| AppError::validation("invalid money"))?;
    let cents_str = parts.next().unwrap_or("0");
    let cents = match cents_str.len() {
        0 => 0,
        1 => cents_str.parse::<i32>().unwrap_or(0) * 10,
        _ => cents_str[..2].parse::<i32>().unwrap_or(0),
    };
    Ok(yuan * 100 + cents)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caidou_sign() {
        let mut params = BTreeMap::new();
        params.insert("pid".to_string(), "1000".to_string());
        params.insert("type".to_string(), "alipay".to_string());
        params.insert("out_trade_no".to_string(), "20240101123456".to_string());
        params.insert("name".to_string(), "test".to_string());
        params.insert("money".to_string(), "100.00".to_string());
        params.insert("sign_type".to_string(), "MD5".to_string());
        params.insert("empty".to_string(), "".to_string());
        let actual = caidou_sign(&params, "secret");
        let raw = "money=100.00&name=test&out_trade_no=20240101123456&pid=1000&type=alipaysecret";
        assert_eq!(actual, format!("{:x}", md5::compute(raw)));
    }

    #[test]
    fn test_money_to_cents() {
        assert_eq!(money_to_cents("1").unwrap(), 100);
        assert_eq!(money_to_cents("1.2").unwrap(), 120);
        assert_eq!(money_to_cents("1.23").unwrap(), 123);
        assert_eq!(cents_to_money(123), "1.23");
    }
}
