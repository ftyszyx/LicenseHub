use crate::apis::payment_handler::OrderStatus;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::Utc;
use data_model::{apps, license_plans, orders};
use salvo::prelude::*;
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder, QuerySelect, Statement};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DashboardRecentOrder {
    pub id: i32,
    pub order_no: String,
    pub plan_name: Option<String>,
    pub app_name: Option<String>,
    pub amount_cents: i32,
    pub status: OrderStatus,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_revenue_cents: i64,
    pub total_orders: u64,
    pub total_users: u64,
    pub new_orders_today: u64,
    pub pending_orders: u64,
    pub delivered_orders: u64,
    pub failed_orders: u64,
    pub active_products: u64,
    pub recent_orders: Vec<DashboardRecentOrder>,
}

#[handler]
pub async fn get_dashboard_stats(
    depot: &mut Depot,
) -> Result<ApiResponse<DashboardStats>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    Ok(ApiResponse::success(get_dashboard_stats_impl(state).await?))
}

pub async fn get_dashboard_stats_impl(state: &AppState) -> Result<DashboardStats, AppError> {
    let today_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::validation("invalid current date"))?
        .and_utc()
        .fixed_offset();

    let delivered_status = i16::from(OrderStatus::Delivered);
    let pending_status = i16::from(OrderStatus::Pending);
    let failed_status = i16::from(OrderStatus::Failed);

    let summary = fetch_dashboard_summary(
        state,
        delivered_status,
        pending_status,
        failed_status,
        today_start,
    )
    .await?;

    let recent_rows = orders::Entity::find()
        .find_also_related(license_plans::Entity)
        .find_also_related(apps::Entity)
        .order_by_desc(orders::Column::CreatedAt)
        .limit(8)
        .all(&state.db)
        .await?;
    let recent_orders = recent_rows
        .into_iter()
        .map(|(order, plan, app)| DashboardRecentOrder {
            id: order.id,
            order_no: order.order_no,
            plan_name: plan.map(|plan| plan.name),
            app_name: app.map(|app| app.name),
            amount_cents: order.amount_cents,
            status: OrderStatus::from(order.status),
            created_at: order.created_at,
        })
        .collect();

    Ok(DashboardStats {
        total_revenue_cents: summary.total_revenue_cents,
        total_orders: summary.total_orders,
        total_users: summary.total_users,
        new_orders_today: summary.new_orders_today,
        pending_orders: summary.pending_orders,
        delivered_orders: summary.delivered_orders,
        failed_orders: summary.failed_orders,
        active_products: summary.active_products,
        recent_orders,
    })
}

struct DashboardSummaryRow {
    total_revenue_cents: i64,
    total_orders: u64,
    total_users: u64,
    new_orders_today: u64,
    pending_orders: u64,
    delivered_orders: u64,
    failed_orders: u64,
    active_products: u64,
}

async fn fetch_dashboard_summary(
    state: &AppState,
    delivered_status: i16,
    pending_status: i16,
    failed_status: i16,
    today_start: chrono::DateTime<chrono::FixedOffset>,
) -> Result<DashboardSummaryRow, AppError> {
    let row = state
        .db
        .query_one(Statement::from_sql_and_values(
            state.db.get_database_backend(),
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN "status" = $1 THEN "amount_cents" ELSE 0 END), 0)::BIGINT AS total_revenue_cents,
                COUNT(*)::BIGINT AS total_orders,
                COUNT(*) FILTER (WHERE "created_at" >= $4)::BIGINT AS new_orders_today,
                COUNT(*) FILTER (WHERE "status" = $2)::BIGINT AS pending_orders,
                COUNT(*) FILTER (WHERE "status" = $1)::BIGINT AS delivered_orders,
                COUNT(*) FILTER (WHERE "status" = $3)::BIGINT AS failed_orders,
                (SELECT COUNT(*)::BIGINT FROM "users") AS total_users,
                (SELECT COUNT(*)::BIGINT FROM "license_plans" WHERE "status" = 1) AS active_products
            FROM "orders"
            "#,
            vec![
                delivered_status.into(),
                pending_status.into(),
                failed_status.into(),
                today_start.into(),
            ],
        ))
        .await?
        .ok_or_else(|| AppError::InternalError {
            message: "dashboard summary query returned no rows".to_string(),
        })?;

    Ok(DashboardSummaryRow {
        total_revenue_cents: row.try_get("", "total_revenue_cents")?,
        total_orders: i64_to_u64(row.try_get("", "total_orders")?),
        total_users: i64_to_u64(row.try_get("", "total_users")?),
        new_orders_today: i64_to_u64(row.try_get("", "new_orders_today")?),
        pending_orders: i64_to_u64(row.try_get("", "pending_orders")?),
        delivered_orders: i64_to_u64(row.try_get("", "delivered_orders")?),
        failed_orders: i64_to_u64(row.try_get("", "failed_orders")?),
        active_products: i64_to_u64(row.try_get("", "active_products")?),
    })
}

fn i64_to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or_default()
}
