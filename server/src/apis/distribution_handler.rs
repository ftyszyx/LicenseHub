use crate::apis::auth_middleware::Claims;
use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::apis::system_settings_handler::get_distribution_settings;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::Utc;
use data_model::{
    distribution_commission_adjustment_offsets, distribution_commission_adjustments,
    distribution_commissions, distribution_settlement_items, distribution_settlement_proofs,
    distribution_settlements, orders, users,
};
use salvo::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE, HeaderValue};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, IntoActiveModel,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

pub const COMMISSION_STATUS_PENDING: i16 = 0;
pub const COMMISSION_STATUS_AVAILABLE: i16 = 1;
pub const COMMISSION_STATUS_LOCKED: i16 = 2;
pub const COMMISSION_STATUS_SETTLED: i16 = 3;
pub const COMMISSION_STATUS_CANCELLED: i16 = 4;
pub const COMMISSION_STATUS_OFFSET: i16 = 5;

pub const SETTLEMENT_STATUS_PENDING: i16 = 0;
pub const SETTLEMENT_STATUS_PAID: i16 = 1;
pub const SETTLEMENT_STATUS_REJECTED: i16 = 2;

const ADJUSTMENT_STATUS_PENDING: i16 = 0;
const ADJUSTMENT_STATUS_PARTIAL: i16 = 1;
const ADJUSTMENT_STATUS_OFFSET: i16 = 2;
const REFUND_ADJUSTMENT_REASON: &str = "order_refunded";
const MAX_PROOF_SIZE: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlipayAccountInfo {
    pub account: String,
    pub real_name: String,
}

#[derive(Debug, Serialize)]
pub struct DistributionSummary {
    pub referral_code: String,
    pub commission_rate_bps: i32,
    pub pending_amount_cents: i64,
    pub available_amount_cents: i64,
    pub locked_amount_cents: i64,
    pub settled_amount_cents: i64,
    pub adjustment_debt_cents: i64,
    pub min_withdraw_cents: i32,
    pub settlement_account: Option<AlipayAccountInfo>,
    pub order_count: i64,
    pub sales_amount_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct CommissionInfo {
    pub id: i64,
    pub order_id: i32,
    pub order_no: String,
    pub order_time: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub user_id: i32,
    pub username: Option<String>,
    pub order_amount_cents: i32,
    pub commission_rate_bps: i32,
    pub commission_amount_cents: i32,
    pub available_amount_cents: i32,
    pub locked_amount_cents: i32,
    pub settled_amount_cents: i32,
    pub cancelled_amount_cents: i32,
    pub adjustment_amount_cents: i32,
    pub status: i16,
    pub available_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct SettlementInfo {
    pub id: i64,
    pub settlement_no: String,
    pub user_id: i32,
    pub username: Option<String>,
    pub amount_cents: i32,
    pub status: i16,
    pub settlement_account: AlipayAccountInfo,
    pub payment_reference: Option<String>,
    pub payment_proof_file_name: Option<String>,
    pub payment_proof_content_type: Option<String>,
    pub payment_proof_size: Option<i64>,
    pub reject_reason: Option<String>,
    pub requested_at: chrono::DateTime<chrono::FixedOffset>,
    pub reviewed_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub paid_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub reviewed_by: Option<i32>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct AdjustmentInfo {
    pub id: i64,
    pub user_id: i32,
    pub username: Option<String>,
    pub order_id: i32,
    pub order_no: String,
    pub original_commission_id: i64,
    pub amount_cents: i32,
    pub offset_amount_cents: i32,
    pub remaining_amount_cents: i32,
    pub reason: String,
    pub status: i16,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CommissionListParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub status: Option<i16>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SettlementListParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub status: Option<i16>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct AdjustmentListParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub status: Option<i16>,
    pub user_id: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSettlementReq {
    pub amount_cents: Option<i32>,
    #[validate(length(min = 1, max = 255))]
    pub alipay_account: String,
    #[validate(length(min = 1, max = 100))]
    pub real_name: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RejectSettlementReq {
    #[validate(length(min = 1, max = 1000))]
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct SettlementPaymentProof {
    pub payment_reference: String,
    pub file_name: String,
    pub content_type: String,
    pub content: Vec<u8>,
}

async fn require_distribution(state: &AppState) -> Result<i32, AppError> {
    let settings = get_distribution_settings(state).await?;
    if !settings.enabled {
        return Err(AppError::business_logic(
            "DISTRIBUTION_NOT_AVAILABLE",
            "distribution feature is not available",
        ));
    }
    Ok(settings.default_rate_bps)
}

async fn release_available(state: &AppState) -> Result<(), AppError> {
    let now = Utc::now().fixed_offset();
    distribution_commissions::Entity::update_many()
        .col_expr(
            distribution_commissions::Column::Status,
            sea_orm::sea_query::Expr::value(COMMISSION_STATUS_AVAILABLE),
        )
        .filter(distribution_commissions::Column::Status.eq(COMMISSION_STATUS_PENDING))
        .filter(distribution_commissions::Column::AvailableAt.lte(now))
        .exec(&state.db)
        .await?;
    Ok(())
}

async fn lock_distribution_user(
    tx: &DatabaseTransaction,
    user_id: i32,
) -> Result<users::Model, AppError> {
    users::Entity::find_by_id(user_id)
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("user", Some(user_id)))
}

async fn lock_settlement_after_user(
    tx: &DatabaseTransaction,
    settlement_id: i64,
) -> Result<distribution_settlements::Model, AppError> {
    let settlement = distribution_settlements::Entity::find_by_id(settlement_id)
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_settlements", None))?;
    lock_distribution_user(tx, settlement.user_id).await?;
    distribution_settlements::Entity::find_by_id(settlement_id)
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_settlements", None))
}

fn allocated_amount(commission: &distribution_commissions::Model) -> i32 {
    commission.locked_amount_cents
        + commission.settled_amount_cents
        + commission.cancelled_amount_cents
        + commission.adjustment_amount_cents
}

fn raw_available_amount(commission: &distribution_commissions::Model) -> i32 {
    if commission.status != COMMISSION_STATUS_AVAILABLE {
        return 0;
    }
    (commission.commission_amount_cents - allocated_amount(commission)).max(0)
}

fn status_after_allocation(commission: &distribution_commissions::Model) -> i16 {
    let remaining = commission.commission_amount_cents - allocated_amount(commission);
    if remaining > 0 {
        COMMISSION_STATUS_AVAILABLE
    } else if commission.locked_amount_cents > 0 {
        COMMISSION_STATUS_LOCKED
    } else if commission.settled_amount_cents > 0 {
        COMMISSION_STATUS_SETTLED
    } else if commission.adjustment_amount_cents > 0 {
        COMMISSION_STATUS_OFFSET
    } else {
        COMMISSION_STATUS_CANCELLED
    }
}

fn normalize_account(account: String, real_name: String) -> Result<AlipayAccountInfo, AppError> {
    let account = account.trim().to_string();
    let real_name = real_name.trim().to_string();
    if account.is_empty() || real_name.is_empty() {
        return Err(AppError::validation(
            "alipay_account and real_name are required",
        ));
    }
    if account.chars().count() > 255 || real_name.chars().count() > 100 {
        return Err(AppError::validation("settlement account is too long"));
    }
    Ok(AlipayAccountInfo { account, real_name })
}

fn account_from_json(value: &serde_json::Value) -> Result<AlipayAccountInfo, AppError> {
    serde_json::from_value(value.clone())
        .map_err(|_| AppError::validation("invalid settlement account snapshot"))
}

fn settlement_info(
    settlement: distribution_settlements::Model,
    username: Option<String>,
) -> Result<SettlementInfo, AppError> {
    Ok(SettlementInfo {
        id: settlement.id,
        settlement_no: settlement.settlement_no,
        user_id: settlement.user_id,
        username,
        amount_cents: settlement.amount_cents,
        status: settlement.status,
        settlement_account: account_from_json(&settlement.settlement_account)?,
        payment_reference: settlement.payment_reference,
        payment_proof_file_name: settlement.payment_proof_file_name,
        payment_proof_content_type: settlement.payment_proof_content_type,
        payment_proof_size: settlement.payment_proof_size,
        reject_reason: settlement.reject_reason,
        requested_at: settlement.requested_at,
        reviewed_at: settlement.reviewed_at,
        paid_at: settlement.paid_at,
        reviewed_by: settlement.reviewed_by,
        created_at: settlement.created_at,
    })
}

async fn outstanding_debt(tx: &DatabaseTransaction, user_id: i32) -> Result<i64, AppError> {
    let rows = distribution_commission_adjustments::Entity::find()
        .filter(distribution_commission_adjustments::Column::UserId.eq(user_id))
        .filter(
            distribution_commission_adjustments::Column::Status
                .is_in([ADJUSTMENT_STATUS_PENDING, ADJUSTMENT_STATUS_PARTIAL]),
        )
        .all(tx)
        .await?;
    Ok(rows
        .iter()
        .map(|row| (-row.amount_cents - row.offset_amount_cents).max(0) as i64)
        .sum())
}

async fn apply_outstanding_adjustments_with_user_lock(
    tx: &DatabaseTransaction,
    user_id: i32,
) -> Result<(), AppError> {
    let adjustments = distribution_commission_adjustments::Entity::find()
        .filter(distribution_commission_adjustments::Column::UserId.eq(user_id))
        .filter(
            distribution_commission_adjustments::Column::Status
                .is_in([ADJUSTMENT_STATUS_PENDING, ADJUSTMENT_STATUS_PARTIAL]),
        )
        .order_by_asc(distribution_commission_adjustments::Column::CreatedAt)
        .order_by_asc(distribution_commission_adjustments::Column::Id)
        .lock_exclusive()
        .all(tx)
        .await?;
    if adjustments.is_empty() {
        return Ok(());
    }

    let mut commissions = distribution_commissions::Entity::find()
        .filter(distribution_commissions::Column::UserId.eq(user_id))
        .filter(distribution_commissions::Column::Status.eq(COMMISSION_STATUS_AVAILABLE))
        .order_by_asc(distribution_commissions::Column::AvailableAt)
        .order_by_asc(distribution_commissions::Column::Id)
        .lock_exclusive()
        .all(tx)
        .await?;
    let now = Utc::now().fixed_offset();
    let mut commission_index = 0_usize;

    for adjustment in adjustments {
        let mut remaining_debt = (-adjustment.amount_cents - adjustment.offset_amount_cents).max(0);
        let mut added_offset = 0_i32;
        while remaining_debt > 0 && commission_index < commissions.len() {
            let available = raw_available_amount(&commissions[commission_index]);
            if available <= 0 {
                commission_index += 1;
                continue;
            }
            let amount = available.min(remaining_debt);
            let mut updated = commissions[commission_index].clone();
            updated.adjustment_amount_cents += amount;
            updated.status = status_after_allocation(&updated);
            updated.updated_at = now;
            let mut active = updated.clone().into_active_model();
            active.adjustment_amount_cents = Set(updated.adjustment_amount_cents);
            active.status = Set(updated.status);
            active.updated_at = Set(now);
            commissions[commission_index] = active.update(tx).await?;

            distribution_commission_adjustment_offsets::ActiveModel {
                adjustment_id: Set(adjustment.id),
                commission_id: Set(updated.id),
                amount_cents: Set(amount),
                created_at: Set(now),
                ..Default::default()
            }
            .insert(tx)
            .await?;

            added_offset += amount;
            remaining_debt -= amount;
            if raw_available_amount(&commissions[commission_index]) == 0 {
                commission_index += 1;
            }
        }

        if added_offset > 0 {
            let new_offset = adjustment.offset_amount_cents + added_offset;
            let new_status = if new_offset >= -adjustment.amount_cents {
                ADJUSTMENT_STATUS_OFFSET
            } else {
                ADJUSTMENT_STATUS_PARTIAL
            };
            let mut active = adjustment.into_active_model();
            active.offset_amount_cents = Set(new_offset);
            active.status = Set(new_status);
            active.updated_at = Set(now);
            active.update(tx).await?;
        }
    }
    Ok(())
}

#[handler]
pub async fn my_summary(depot: &mut Depot) -> Result<ApiResponse<DistributionSummary>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let default_rate = require_distribution(state).await?;
    release_available(state).await?;

    let tx = state.db.begin().await?;
    lock_distribution_user(&tx, claims.user_id).await?;
    apply_outstanding_adjustments_with_user_lock(&tx, claims.user_id).await?;
    let debt = outstanding_debt(&tx, claims.user_id).await?;
    tx.commit().await?;

    let user = users::Entity::find_by_id(claims.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("user", Some(claims.user_id)))?;
    let rows = distribution_commissions::Entity::find()
        .filter(distribution_commissions::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?;
    let settings = get_distribution_settings(state).await?;
    let mut pending = 0_i64;
    let mut available = 0_i64;
    let mut locked = 0_i64;
    let mut settled = 0_i64;
    let mut sales = 0_i64;
    for row in &rows {
        sales += row.order_amount_cents as i64;
        if row.status == COMMISSION_STATUS_PENDING {
            pending += row.commission_amount_cents as i64;
        }
        available += raw_available_amount(row) as i64;
        locked += row.locked_amount_cents as i64;
        settled += row.settled_amount_cents as i64;
    }
    Ok(ApiResponse::success(DistributionSummary {
        referral_code: user.referral_code,
        commission_rate_bps: user.commission_rate_bps.unwrap_or(default_rate),
        pending_amount_cents: pending,
        available_amount_cents: available,
        locked_amount_cents: locked,
        settled_amount_cents: settled,
        adjustment_debt_cents: debt,
        min_withdraw_cents: settings.min_withdraw_cents,
        settlement_account: user
            .settlement_account
            .as_ref()
            .map(account_from_json)
            .transpose()?,
        order_count: rows.len() as i64,
        sales_amount_cents: sales,
    }))
}

#[handler]
pub async fn my_commissions(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<CommissionInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    require_distribution(state).await?;
    release_available(state).await?;
    let tx = state.db.begin().await?;
    lock_distribution_user(&tx, claims.user_id).await?;
    apply_outstanding_adjustments_with_user_lock(&tx, claims.user_id).await?;
    tx.commit().await?;
    let mut params = req.parse_queries::<CommissionListParams>()?;
    params.user_id = Some(claims.user_id);
    Ok(ApiResponse::success(list_commissions(state, params).await?))
}

#[handler]
pub async fn admin_commissions(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<CommissionInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    release_available(state).await?;
    Ok(ApiResponse::success(
        list_commissions(state, req.parse_queries()?).await?,
    ))
}

async fn list_commissions(
    state: &AppState,
    params: CommissionListParams,
) -> Result<PagingResponse<CommissionInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = distribution_commissions::Entity::find()
        .find_also_related(orders::Entity)
        .find_also_related(users::Entity)
        .order_by_desc(distribution_commissions::Column::CreatedAt);
    if let Some(status) = params.status {
        query = query.filter(distribution_commissions::Column::Status.eq(status));
    }
    if let Some(user_id) = params.user_id {
        query = query.filter(distribution_commissions::Column::UserId.eq(user_id));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let list = rows
        .into_iter()
        .map(|(commission, order, user)| {
            let (order_no, order_time) = match order {
                Some(order) => (order.order_no, order.paid_at.or(Some(order.created_at))),
                None => (String::new(), None),
            };
            CommissionInfo {
                id: commission.id,
                order_id: commission.order_id,
                order_no,
                order_time,
                user_id: commission.user_id,
                username: user.map(|value| value.username),
                order_amount_cents: commission.order_amount_cents,
                commission_rate_bps: commission.commission_rate_bps,
                commission_amount_cents: commission.commission_amount_cents,
                available_amount_cents: raw_available_amount(&commission),
                locked_amount_cents: commission.locked_amount_cents,
                settled_amount_cents: commission.settled_amount_cents,
                cancelled_amount_cents: commission.cancelled_amount_cents,
                adjustment_amount_cents: commission.adjustment_amount_cents,
                status: commission.status,
                available_at: commission.available_at,
                created_at: commission.created_at,
            }
        })
        .collect();
    Ok(PagingResponse { list, total, page })
}

#[handler]
pub async fn create_settlement(
    depot: &mut Depot,
    req: JsonBody<CreateSettlementReq>,
) -> Result<ApiResponse<SettlementInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let req = req.into_inner();
    req.validate()?;
    Ok(ApiResponse::success(
        create_settlement_impl(state, claims.user_id, req).await?,
    ))
}

pub async fn create_settlement_impl(
    state: &AppState,
    user_id: i32,
    req: CreateSettlementReq,
) -> Result<SettlementInfo, AppError> {
    require_distribution(state).await?;
    release_available(state).await?;
    let settings = get_distribution_settings(state).await?;
    let account = normalize_account(req.alipay_account, req.real_name)?;
    let tx = state.db.begin().await?;

    let user = lock_distribution_user(&tx, user_id).await?;
    let existing = distribution_settlements::Entity::find()
        .filter(distribution_settlements::Column::UserId.eq(user_id))
        .filter(distribution_settlements::Column::Status.eq(SETTLEMENT_STATUS_PENDING))
        .one(&tx)
        .await?;
    if existing.is_some() {
        return Err(AppError::business_logic(
            "WITHDRAWAL_PENDING_EXISTS",
            "a pending withdrawal request already exists",
        ));
    }

    apply_outstanding_adjustments_with_user_lock(&tx, user_id).await?;
    let commissions = distribution_commissions::Entity::find()
        .filter(distribution_commissions::Column::UserId.eq(user_id))
        .filter(distribution_commissions::Column::Status.eq(COMMISSION_STATUS_AVAILABLE))
        .order_by_asc(distribution_commissions::Column::AvailableAt)
        .order_by_asc(distribution_commissions::Column::Id)
        .lock_exclusive()
        .all(&tx)
        .await?;
    let available: i32 = commissions.iter().map(raw_available_amount).sum();
    let amount = req.amount_cents.unwrap_or(available);
    if amount < settings.min_withdraw_cents {
        return Err(AppError::business_logic(
            "WITHDRAWAL_BELOW_MINIMUM",
            format!(
                "withdrawal amount must be at least {} cents",
                settings.min_withdraw_cents
            ),
        ));
    }
    if amount <= 0 || amount > available {
        return Err(AppError::business_logic(
            "WITHDRAWAL_AMOUNT_INVALID",
            "withdrawal amount exceeds available commission",
        ));
    }

    let now = Utc::now().fixed_offset();
    let account_json = serde_json::to_value(&account)
        .map_err(|_| AppError::validation("invalid settlement account"))?;
    let settlement = distribution_settlements::ActiveModel {
        settlement_no: Set(format!("WD{}", uuid::Uuid::new_v4().simple())),
        user_id: Set(user_id),
        amount_cents: Set(amount),
        status: Set(SETTLEMENT_STATUS_PENDING),
        settlement_account: Set(account_json.clone()),
        requested_at: Set(now),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&tx)
    .await?;

    let mut remaining = amount;
    for commission in commissions {
        if remaining <= 0 {
            break;
        }
        let allocated = raw_available_amount(&commission).min(remaining);
        if allocated <= 0 {
            continue;
        }
        let mut updated = commission.clone();
        updated.locked_amount_cents += allocated;
        updated.status = status_after_allocation(&updated);
        updated.updated_at = now;
        let mut active = updated.into_active_model();
        active.locked_amount_cents = Set(commission.locked_amount_cents + allocated);
        active.status = Set(status_after_allocation(&distribution_commissions::Model {
            locked_amount_cents: commission.locked_amount_cents + allocated,
            ..commission.clone()
        }));
        active.updated_at = Set(now);
        active.update(&tx).await?;

        distribution_settlement_items::ActiveModel {
            settlement_id: Set(settlement.id),
            commission_id: Set(commission.id),
            amount_cents: Set(allocated),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(&tx)
        .await?;
        remaining -= allocated;
    }
    if remaining != 0 {
        return Err(AppError::InternalError {
            message: "failed to allocate withdrawal commissions".to_string(),
        });
    }

    let mut user_active = user.into_active_model();
    user_active.settlement_account = Set(Some(account_json));
    user_active.updated_at = Set(now);
    user_active.update(&tx).await?;
    tx.commit().await?;
    get_settlement_info(state, settlement.id).await
}

#[handler]
pub async fn my_settlements(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<SettlementInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    require_distribution(state).await?;
    let mut params = req.parse_queries::<SettlementListParams>()?;
    params.user_id = Some(claims.user_id);
    Ok(ApiResponse::success(list_settlements(state, params).await?))
}

#[handler]
pub async fn admin_settlements(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<SettlementInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    Ok(ApiResponse::success(
        list_settlements(state, req.parse_queries()?).await?,
    ))
}

async fn list_settlements(
    state: &AppState,
    params: SettlementListParams,
) -> Result<PagingResponse<SettlementInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = distribution_settlements::Entity::find()
        .find_also_related(users::Entity)
        .order_by_desc(distribution_settlements::Column::CreatedAt);
    if let Some(status) = params.status {
        query = query.filter(distribution_settlements::Column::Status.eq(status));
    }
    if let Some(user_id) = params.user_id {
        query = query.filter(distribution_settlements::Column::UserId.eq(user_id));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let list = rows
        .into_iter()
        .map(|(settlement, user)| settlement_info(settlement, user.map(|value| value.username)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PagingResponse { list, total, page })
}

async fn get_settlement_info(
    state: &AppState,
    settlement_id: i64,
) -> Result<SettlementInfo, AppError> {
    let (settlement, user) = distribution_settlements::Entity::find_by_id(settlement_id)
        .find_also_related(users::Entity)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_settlements", None))?;
    settlement_info(settlement, user.map(|value| value.username))
}

async fn reject_settlement_with_user_lock(
    tx: &DatabaseTransaction,
    settlement: distribution_settlements::Model,
    operator_user_id: i32,
    reason: String,
) -> Result<distribution_settlements::Model, AppError> {
    if settlement.status == SETTLEMENT_STATUS_REJECTED {
        return Ok(settlement);
    }
    if settlement.status != SETTLEMENT_STATUS_PENDING {
        return Err(AppError::business_logic(
            "SETTLEMENT_NOT_PENDING",
            "only pending withdrawals can be rejected",
        ));
    }
    let reason = reason.trim().to_string();
    if reason.is_empty() {
        return Err(AppError::validation("reject reason is required"));
    }
    let items = distribution_settlement_items::Entity::find()
        .filter(distribution_settlement_items::Column::SettlementId.eq(settlement.id))
        .all(tx)
        .await?;
    let now = Utc::now().fixed_offset();
    for item in items {
        let commission = distribution_commissions::Entity::find_by_id(item.commission_id)
            .lock_exclusive()
            .one(tx)
            .await?
            .ok_or_else(|| AppError::not_found("distribution_commissions", None))?;
        if commission.locked_amount_cents < item.amount_cents {
            return Err(AppError::InternalError {
                message: "settlement commission lock is inconsistent".to_string(),
            });
        }
        let mut updated = commission.clone();
        updated.locked_amount_cents -= item.amount_cents;
        updated.status = status_after_allocation(&updated);
        let mut active = updated.into_active_model();
        active.locked_amount_cents = Set(commission.locked_amount_cents - item.amount_cents);
        active.status = Set(status_after_allocation(&distribution_commissions::Model {
            locked_amount_cents: commission.locked_amount_cents - item.amount_cents,
            ..commission
        }));
        active.updated_at = Set(now);
        active.update(tx).await?;
    }
    let mut active = settlement.into_active_model();
    active.status = Set(SETTLEMENT_STATUS_REJECTED);
    active.reject_reason = Set(Some(reason));
    active.reviewed_at = Set(Some(now));
    active.reviewed_by = Set(Some(operator_user_id));
    active.updated_at = Set(now);
    Ok(active.update(tx).await?)
}

#[handler]
pub async fn reject_settlement(
    depot: &mut Depot,
    id: PathParam<i64>,
    req: JsonBody<RejectSettlementReq>,
) -> Result<ApiResponse<SettlementInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let req = req.into_inner();
    req.validate()?;
    let tx = state.db.begin().await?;
    let settlement = lock_settlement_after_user(&tx, id.into_inner()).await?;
    let settlement =
        reject_settlement_with_user_lock(&tx, settlement, claims.user_id, req.reason).await?;
    tx.commit().await?;
    Ok(ApiResponse::success(
        get_settlement_info(state, settlement.id).await?,
    ))
}

fn validate_payment_proof(
    proof: SettlementPaymentProof,
) -> Result<SettlementPaymentProof, AppError> {
    let payment_reference = proof.payment_reference.trim().to_string();
    if payment_reference.is_empty() || payment_reference.chars().count() > 255 {
        return Err(AppError::validation("payment_reference is required"));
    }
    if proof.content.is_empty() || proof.content.len() as u64 > MAX_PROOF_SIZE {
        return Err(AppError::validation(
            "payment proof must be between 1 byte and 5 MB",
        ));
    }
    let content_type = proof.content_type.to_ascii_lowercase();
    if !matches!(
        content_type.as_str(),
        "image/jpeg" | "image/png" | "image/webp" | "application/pdf"
    ) {
        return Err(AppError::validation(
            "payment proof must be JPG, PNG, WebP, or PDF",
        ));
    }
    let file_name = std::path::Path::new(&proof.file_name)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("payment-proof")
        .chars()
        .take(255)
        .collect::<String>();
    Ok(SettlementPaymentProof {
        payment_reference,
        file_name,
        content_type,
        content: proof.content,
    })
}

pub async fn mark_settlement_paid_impl(
    state: &AppState,
    operator_user_id: i32,
    settlement_id: i64,
    proof: SettlementPaymentProof,
) -> Result<SettlementInfo, AppError> {
    let proof = validate_payment_proof(proof)?;
    let tx = state.db.begin().await?;
    let settlement = lock_settlement_after_user(&tx, settlement_id).await?;
    if settlement.status == SETTLEMENT_STATUS_PAID {
        tx.commit().await?;
        return get_settlement_info(state, settlement_id).await;
    }
    if settlement.status != SETTLEMENT_STATUS_PENDING {
        return Err(AppError::business_logic(
            "SETTLEMENT_NOT_PENDING",
            "only pending withdrawals can be marked paid",
        ));
    }

    let items = distribution_settlement_items::Entity::find()
        .filter(distribution_settlement_items::Column::SettlementId.eq(settlement.id))
        .all(&tx)
        .await?;
    let now = Utc::now().fixed_offset();
    for item in items {
        let commission = distribution_commissions::Entity::find_by_id(item.commission_id)
            .lock_exclusive()
            .one(&tx)
            .await?
            .ok_or_else(|| AppError::not_found("distribution_commissions", None))?;
        if commission.locked_amount_cents < item.amount_cents {
            return Err(AppError::InternalError {
                message: "settlement commission lock is inconsistent".to_string(),
            });
        }
        let mut updated = commission.clone();
        updated.locked_amount_cents -= item.amount_cents;
        updated.settled_amount_cents += item.amount_cents;
        updated.status = status_after_allocation(&updated);
        let mut active = updated.into_active_model();
        active.locked_amount_cents = Set(commission.locked_amount_cents - item.amount_cents);
        active.settled_amount_cents = Set(commission.settled_amount_cents + item.amount_cents);
        active.status = Set(status_after_allocation(&distribution_commissions::Model {
            locked_amount_cents: commission.locked_amount_cents - item.amount_cents,
            settled_amount_cents: commission.settled_amount_cents + item.amount_cents,
            ..commission
        }));
        active.updated_at = Set(now);
        active.update(&tx).await?;
    }

    distribution_settlement_proofs::ActiveModel {
        settlement_id: Set(settlement.id),
        content: Set(proof.content.clone()),
        uploaded_by: Set(operator_user_id),
        created_at: Set(now),
    }
    .insert(&tx)
    .await?;

    let mut active = settlement.into_active_model();
    active.status = Set(SETTLEMENT_STATUS_PAID);
    active.payment_reference = Set(Some(proof.payment_reference));
    active.payment_proof_file_name = Set(Some(proof.file_name));
    active.payment_proof_content_type = Set(Some(proof.content_type));
    active.payment_proof_size = Set(Some(proof.content.len() as i64));
    active.reviewed_at = Set(Some(now));
    active.paid_at = Set(Some(now));
    active.reviewed_by = Set(Some(operator_user_id));
    active.updated_at = Set(now);
    let settlement = active.update(&tx).await?;
    tx.commit().await?;
    get_settlement_info(state, settlement.id).await
}

#[handler]
pub async fn mark_settlement_paid(
    depot: &mut Depot,
    id: PathParam<i64>,
    req: &mut Request,
) -> Result<ApiResponse<SettlementInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let form = req.form_data().await?;
    let payment_reference = form
        .fields
        .get("payment_reference")
        .cloned()
        .unwrap_or_default();
    let file = form
        .files
        .get("proof")
        .ok_or_else(|| AppError::validation("payment proof is required"))?;
    if file.size() > MAX_PROOF_SIZE {
        return Err(AppError::validation("payment proof must not exceed 5 MB"));
    }
    let proof = SettlementPaymentProof {
        payment_reference,
        file_name: file.name().unwrap_or("payment-proof").to_string(),
        content_type: file
            .content_type()
            .map(|value| value.to_string())
            .unwrap_or_default(),
        content: tokio::fs::read(file.path())
            .await
            .map_err(|error| AppError::InternalError {
                message: format!("failed to read payment proof: {error}"),
            })?,
    };
    Ok(ApiResponse::success(
        mark_settlement_paid_impl(state, claims.user_id, id.into_inner(), proof).await?,
    ))
}

async fn write_settlement_proof(
    state: &AppState,
    settlement_id: i64,
    res: &mut Response,
) -> Result<(), AppError> {
    let settlement = distribution_settlements::Entity::find_by_id(settlement_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_settlements", None))?;
    let proof = distribution_settlement_proofs::Entity::find_by_id(settlement_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_settlement_proofs", None))?;
    let content_type = settlement
        .payment_proof_content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let extension = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "application/pdf" => "pdf",
        _ => "bin",
    };
    res.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&content_type)
            .map_err(|_| AppError::validation("invalid proof content type"))?,
    );
    res.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "inline; filename=\"payment-proof-{settlement_id}.{extension}\""
        ))
        .map_err(|_| AppError::validation("invalid proof file name"))?,
    );
    res.write_body(proof.content)
        .map_err(|error| AppError::InternalError {
            message: format!("failed to write payment proof: {error}"),
        })?;
    Ok(())
}

#[handler]
pub async fn settlement_proof(
    depot: &mut Depot,
    id: PathParam<i64>,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    write_settlement_proof(state, id.into_inner(), res).await
}

#[handler]
pub async fn my_settlement_proof(
    depot: &mut Depot,
    id: PathParam<i64>,
    res: &mut Response,
) -> Result<(), AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    require_distribution(state).await?;
    let settlement_id = id.into_inner();
    let owned = distribution_settlements::Entity::find_by_id(settlement_id)
        .filter(distribution_settlements::Column::UserId.eq(claims.user_id))
        .one(&state.db)
        .await?;
    if owned.is_none() {
        return Err(AppError::not_found("distribution_settlements", None));
    }
    write_settlement_proof(state, settlement_id, res).await
}

#[handler]
pub async fn my_adjustments(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<AdjustmentInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    require_distribution(state).await?;
    let mut params = req.parse_queries::<AdjustmentListParams>()?;
    params.user_id = Some(claims.user_id);
    Ok(ApiResponse::success(list_adjustments(state, params).await?))
}

#[handler]
pub async fn admin_adjustments(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<AdjustmentInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    Ok(ApiResponse::success(
        list_adjustments(state, req.parse_queries()?).await?,
    ))
}

async fn list_adjustments(
    state: &AppState,
    params: AdjustmentListParams,
) -> Result<PagingResponse<AdjustmentInfo>, AppError> {
    let (page, page_size) = params.pagination.resolve()?;
    let mut query = distribution_commission_adjustments::Entity::find()
        .find_also_related(orders::Entity)
        .find_also_related(users::Entity)
        .order_by_desc(distribution_commission_adjustments::Column::CreatedAt);
    if let Some(status) = params.status {
        query = query.filter(distribution_commission_adjustments::Column::Status.eq(status));
    }
    if let Some(user_id) = params.user_id {
        query = query.filter(distribution_commission_adjustments::Column::UserId.eq(user_id));
    }
    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let rows = paginator.fetch_page(page - 1).await?;
    let list = rows
        .into_iter()
        .map(|(adjustment, order, user)| AdjustmentInfo {
            id: adjustment.id,
            user_id: adjustment.user_id,
            username: user.map(|value| value.username),
            order_id: adjustment.order_id,
            order_no: order.map(|value| value.order_no).unwrap_or_default(),
            original_commission_id: adjustment.original_commission_id,
            amount_cents: adjustment.amount_cents,
            offset_amount_cents: adjustment.offset_amount_cents,
            remaining_amount_cents: (-adjustment.amount_cents - adjustment.offset_amount_cents)
                .max(0),
            reason: adjustment.reason,
            status: adjustment.status,
            created_at: adjustment.created_at,
        })
        .collect();
    Ok(PagingResponse { list, total, page })
}

pub(crate) async fn handle_commission_refund(
    tx: &DatabaseTransaction,
    commission: distribution_commissions::Model,
    operator_user_id: i32,
) -> Result<(), AppError> {
    lock_distribution_user(tx, commission.user_id).await?;
    let commission = distribution_commissions::Entity::find_by_id(commission.id)
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_commissions", None))?;
    let linked = distribution_settlement_items::Entity::find()
        .filter(distribution_settlement_items::Column::CommissionId.eq(commission.id))
        .find_also_related(distribution_settlements::Entity)
        .all(tx)
        .await?;
    for (_, settlement) in linked {
        let Some(settlement) = settlement else {
            continue;
        };
        if settlement.status != SETTLEMENT_STATUS_PENDING {
            continue;
        }
        let settlement = distribution_settlements::Entity::find_by_id(settlement.id)
            .lock_exclusive()
            .one(tx)
            .await?
            .ok_or_else(|| AppError::not_found("distribution_settlements", None))?;
        reject_settlement_with_user_lock(
            tx,
            settlement,
            operator_user_id,
            "关联订单退款，提现申请已自动驳回".to_string(),
        )
        .await?;
    }

    // Rejecting a pending settlement releases locked commission amounts. Reload the
    // row so the refund calculation uses the post-rejection allocation state.
    let commission = distribution_commissions::Entity::find_by_id(commission.id)
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or_else(|| AppError::not_found("distribution_commissions", None))?;

    let exposed_amount = commission.settled_amount_cents + commission.adjustment_amount_cents;
    let cancelled_amount =
        (commission.commission_amount_cents - exposed_amount - commission.locked_amount_cents)
            .max(0);
    let now = Utc::now().fixed_offset();
    let mut updated = commission.clone();
    updated.cancelled_amount_cents = cancelled_amount;
    updated.locked_amount_cents = 0;
    updated.cancel_reason = Some(REFUND_ADJUSTMENT_REASON.to_string());
    updated.status = status_after_allocation(&updated);
    let mut active = updated.into_active_model();
    active.cancelled_amount_cents = Set(cancelled_amount);
    active.locked_amount_cents = Set(0);
    active.cancel_reason = Set(Some(REFUND_ADJUSTMENT_REASON.to_string()));
    active.status = Set(status_after_allocation(&distribution_commissions::Model {
        cancelled_amount_cents: cancelled_amount,
        locked_amount_cents: 0,
        cancel_reason: Some(REFUND_ADJUSTMENT_REASON.to_string()),
        ..commission.clone()
    }));
    active.updated_at = Set(now);
    active.update(tx).await?;

    if exposed_amount > 0 {
        distribution_commission_adjustments::ActiveModel {
            user_id: Set(commission.user_id),
            order_id: Set(commission.order_id),
            original_commission_id: Set(commission.id),
            amount_cents: Set(-exposed_amount),
            offset_amount_cents: Set(0),
            reason: Set(REFUND_ADJUSTMENT_REASON.to_string()),
            status: Set(ADJUSTMENT_STATUS_PENDING),
            operator_user_id: Set(operator_user_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(tx)
        .await?;
        apply_outstanding_adjustments_with_user_lock(tx, commission.user_id).await?;
    }
    Ok(())
}

pub(crate) fn new_commission_active_model(
    order_id: i32,
    user_id: i32,
    order_amount_cents: i32,
    commission_rate_bps: i32,
    commission_amount_cents: i32,
    available_at: chrono::DateTime<chrono::FixedOffset>,
    now: chrono::DateTime<chrono::FixedOffset>,
) -> distribution_commissions::ActiveModel {
    distribution_commissions::ActiveModel {
        order_id: Set(order_id),
        user_id: Set(user_id),
        order_amount_cents: Set(order_amount_cents),
        commission_rate_bps: Set(commission_rate_bps),
        commission_amount_cents: Set(commission_amount_cents),
        status: Set(COMMISSION_STATUS_PENDING),
        available_at: Set(Some(available_at)),
        locked_amount_cents: Set(0),
        settled_amount_cents: Set(0),
        cancelled_amount_cents: Set(0),
        adjustment_amount_cents: Set(0),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
}
