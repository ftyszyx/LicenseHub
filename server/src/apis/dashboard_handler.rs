use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::apis::payment_handler::OrderStatus;
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::convert::from_str_optional;
use chrono::{Datelike, Duration, FixedOffset, Months, NaiveDate, TimeZone, Utc};
use data_model::{apps, license_plans, orders};
use salvo::prelude::*;
use sea_orm::{ConnectionTrait, EntityTrait, QueryOrder, QuerySelect, Statement};
use serde::{Deserialize, Serialize};

const DAILY_PERIODS: i64 = 30;
const HOURLY_PERIODS: i64 = 24;
const MONTHLY_PERIODS: u32 = 12;
const YEARLY_PERIODS: u32 = 5;
const MAX_HOURLY_DAYS: i64 = 366;
const MAX_DAILY_PERIODS: i64 = 3660;
const MAX_MONTHLY_PERIODS: i32 = 600;
const MAX_YEARLY_PERIODS: i32 = 100;

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

#[derive(Debug, Deserialize, Default)]
pub struct DashboardTrendParams {
    pub group_by: Option<String>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub app_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DashboardTrendPoint {
    pub period: String,
    pub revenue_cents: i64,
    pub order_count: u64,
    pub new_buyer_order_count: u64,
    pub returning_buyer_order_count: u64,
    pub unidentified_buyer_order_count: u64,
}

#[derive(Debug, Serialize)]
pub struct DashboardTrend {
    pub points: Vec<DashboardTrendPoint>,
    pub apps: Vec<DashboardTrendApp>,
}

#[derive(Debug, Serialize)]
pub struct DashboardTrendApp {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DashboardReturningOrdersParams {
    pub group_by: Option<String>,
    pub period: String,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub app_id: Option<i32>,
    #[serde(flatten)]
    pub pagination: ListParamsReq,
}

#[derive(Debug, Serialize)]
pub struct DashboardReturningOrder {
    pub order_no: String,
    pub provider_trade_no: Option<String>,
    pub first_order_no: String,
    pub purchase_number: u64,
    pub app_id: i32,
    pub app_name: String,
    pub amount_cents: i32,
    pub provider: String,
    pub pay_type: String,
    pub provider_buyer_id: String,
    pub buyer_user_id: Option<i32>,
    pub buyer_username: Option<String>,
    pub buyer_user_email: Option<String>,
    pub buyer_email: Option<String>,
    pub paid_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Clone, Copy)]
enum TrendGroupBy {
    Hour,
    Day,
    Month,
    Year,
}

impl TrendGroupBy {
    fn parse(value: Option<&str>) -> Result<Self, AppError> {
        match value.unwrap_or("day") {
            "hour" => Ok(Self::Hour),
            "day" => Ok(Self::Day),
            "month" => Ok(Self::Month),
            "year" => Ok(Self::Year),
            _ => Err(AppError::validation(
                "group_by must be hour, day, month, or year",
            )),
        }
    }
}

#[handler]
pub async fn get_dashboard_stats(
    depot: &mut Depot,
) -> Result<ApiResponse<DashboardStats>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    Ok(ApiResponse::success(get_dashboard_stats_impl(state).await?))
}

#[handler]
pub async fn get_dashboard_trend(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<DashboardTrend>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<DashboardTrendParams>()?;
    Ok(ApiResponse::success(
        get_dashboard_trend_impl(state, params).await?,
    ))
}

#[handler]
pub async fn list_dashboard_returning_orders(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<DashboardReturningOrder>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<DashboardReturningOrdersParams>()?;
    Ok(ApiResponse::success(
        list_dashboard_returning_orders_impl(state, params).await?,
    ))
}

pub async fn get_dashboard_stats_impl(state: &AppState) -> Result<DashboardStats, AppError> {
    let timezone = business_timezone()?;
    let local_midnight = Utc::now()
        .with_timezone(&timezone)
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| AppError::validation("invalid current date"))?;
    let today_start = timezone
        .from_local_datetime(&local_midnight)
        .single()
        .ok_or_else(|| AppError::validation("invalid start of day"))?;

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

pub async fn get_dashboard_trend_impl(
    state: &AppState,
    params: DashboardTrendParams,
) -> Result<DashboardTrend, AppError> {
    if matches!(params.app_id, Some(app_id) if app_id < 1) {
        return Err(AppError::validation("app_id must be greater than 0"));
    }

    let group_by = TrendGroupBy::parse(params.group_by.as_deref())?;
    let periods = trend_periods(
        group_by,
        params.start_date.as_deref(),
        params.end_date.as_deref(),
    )?;
    let start = periods
        .first()
        .ok_or_else(|| AppError::validation("trend period is empty"))?;
    let end = periods
        .last()
        .ok_or_else(|| AppError::validation("trend period is empty"))?;
    let (query_start, query_end) = match group_by {
        TrendGroupBy::Hour => (start.clone(), end.clone()),
        TrendGroupBy::Day => (start.clone(), end.clone()),
        TrendGroupBy::Month => (format!("{start}-01"), format!("{end}-01")),
        TrendGroupBy::Year => (format!("{start}-01-01"), format!("{end}-01-01")),
    };
    let (period_expression, range_expression) = match group_by {
        TrendGroupBy::Hour => (
            r#"TO_CHAR(DATE_TRUNC('hour', "event_time"), 'YYYY-MM-DD HH24:00')"#,
            r#""event_time" >= $2::TIMESTAMP
              AND "event_time" < $3::TIMESTAMP + INTERVAL '1 hour'"#,
        ),
        TrendGroupBy::Day => (
            r#"TO_CHAR("event_time", 'YYYY-MM-DD')"#,
            r#""event_time"::DATE BETWEEN $2::DATE AND $3::DATE"#,
        ),
        TrendGroupBy::Month => (
            r#"TO_CHAR(DATE_TRUNC('month', "event_time"), 'YYYY-MM')"#,
            r#"DATE_TRUNC('month', "event_time")::DATE BETWEEN $2::DATE AND $3::DATE"#,
        ),
        TrendGroupBy::Year => (
            r#"TO_CHAR(DATE_TRUNC('year', "event_time"), 'YYYY')"#,
            r#"DATE_TRUNC('year', "event_time")::DATE BETWEEN $2::DATE AND $3::DATE"#,
        ),
    };
    let sql = format!(
        r#"
        WITH classified_orders AS (
            SELECT
                "id",
                "amount_cents",
                "provider_buyer_id",
                TIMEZONE('Asia/Shanghai', COALESCE("paid_at", "created_at")) AS "event_time",
                CASE
                    WHEN "provider_buyer_id" IS NULL THEN NULL
                    ELSE ROW_NUMBER() OVER (
                        PARTITION BY "provider", "provider_buyer_id"
                        ORDER BY COALESCE("paid_at", "created_at"), "id"
                    )
                END AS "buyer_order_number"
            FROM "orders"
            WHERE "status" = $1
              AND ($4::INTEGER IS NULL OR "app_id" = $4)
        )
        SELECT
            {period_expression} AS period,
            COALESCE(SUM("amount_cents"), 0)::BIGINT AS revenue_cents,
            COUNT(*)::BIGINT AS order_count,
            COUNT(*) FILTER (
                WHERE "provider_buyer_id" IS NOT NULL AND "buyer_order_number" = 1
            )::BIGINT AS new_buyer_order_count,
            COUNT(*) FILTER (
                WHERE "provider_buyer_id" IS NOT NULL AND "buyer_order_number" > 1
            )::BIGINT AS returning_buyer_order_count,
            COUNT(*) FILTER (
                WHERE "provider_buyer_id" IS NULL
            )::BIGINT AS unidentified_buyer_order_count
        FROM classified_orders
        WHERE {range_expression}
        GROUP BY period
        ORDER BY period
        "#
    );

    let rows = state
        .db
        .query_all(Statement::from_sql_and_values(
            state.db.get_database_backend(),
            sql,
            vec![
                i16::from(OrderStatus::Delivered).into(),
                query_start.into(),
                query_end.into(),
                params.app_id.into(),
            ],
        ))
        .await?;
    let values = rows
        .into_iter()
        .map(|row| {
            Ok((
                row.try_get::<String>("", "period")?,
                (
                    row.try_get::<i64>("", "revenue_cents")?,
                    row.try_get::<i64>("", "order_count")?,
                    row.try_get::<i64>("", "new_buyer_order_count")?,
                    row.try_get::<i64>("", "returning_buyer_order_count")?,
                    row.try_get::<i64>("", "unidentified_buyer_order_count")?,
                ),
            ))
        })
        .collect::<Result<std::collections::HashMap<_, _>, sea_orm::DbErr>>()?;

    let apps = apps::Entity::find()
        .order_by_asc(apps::Column::SortOrder)
        .order_by_asc(apps::Column::Id)
        .all(&state.db)
        .await?
        .into_iter()
        .map(|app| DashboardTrendApp {
            id: app.id,
            name: app.name,
        })
        .collect();

    Ok(DashboardTrend {
        points: periods
            .into_iter()
            .map(|period| {
                let (
                    revenue_cents,
                    order_count,
                    new_buyer_order_count,
                    returning_buyer_order_count,
                    unidentified_buyer_order_count,
                ) = values.get(&period).copied().unwrap_or_default();
                DashboardTrendPoint {
                    period,
                    revenue_cents,
                    order_count: i64_to_u64(order_count),
                    new_buyer_order_count: i64_to_u64(new_buyer_order_count),
                    returning_buyer_order_count: i64_to_u64(returning_buyer_order_count),
                    unidentified_buyer_order_count: i64_to_u64(unidentified_buyer_order_count),
                }
            })
            .collect(),
        apps,
    })
}

pub async fn list_dashboard_returning_orders_impl(
    state: &AppState,
    params: DashboardReturningOrdersParams,
) -> Result<PagingResponse<DashboardReturningOrder>, AppError> {
    if matches!(params.app_id, Some(app_id) if app_id < 1) {
        return Err(AppError::validation("app_id must be greater than 0"));
    }
    let group_by = TrendGroupBy::parse(params.group_by.as_deref())?;
    let (query_start, query_end) = returning_order_period_bounds(group_by, &params.period)?;
    let (page, page_size) = params.pagination.resolve()?;
    let offset = (page - 1)
        .checked_mul(page_size)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| AppError::validation("pagination offset is too large"))?;
    let limit =
        i64::try_from(page_size).map_err(|_| AppError::validation("page_size is too large"))?;

    let classified_orders = r#"
        WITH classified_orders AS (
            SELECT
                "id",
                "order_no",
                "provider_trade_no",
                "app_id",
                "amount_cents",
                "provider",
                "pay_type",
                "provider_buyer_id",
                "buyer_user_id",
                "buyer_email",
                "paid_at",
                "created_at",
                TIMEZONE('Asia/Shanghai', COALESCE("paid_at", "created_at")) AS "event_time",
                ROW_NUMBER() OVER (
                    PARTITION BY "provider", "provider_buyer_id"
                    ORDER BY COALESCE("paid_at", "created_at"), "id"
                ) AS "buyer_order_number",
                FIRST_VALUE("order_no") OVER (
                    PARTITION BY "provider", "provider_buyer_id"
                    ORDER BY COALESCE("paid_at", "created_at"), "id"
                ) AS "first_order_no"
            FROM "orders"
            WHERE "status" = $1
              AND "provider_buyer_id" IS NOT NULL
              AND ($2::INTEGER IS NULL OR "app_id" = $2)
        )
    "#;
    let count_sql = format!(
        r#"
        {classified_orders}
        SELECT COUNT(*)::BIGINT AS "total"
        FROM classified_orders
        WHERE "buyer_order_number" > 1
          AND "event_time" >= $3::TIMESTAMP
          AND "event_time" < $4::TIMESTAMP
        "#
    );
    let common_values = vec![
        i16::from(OrderStatus::Delivered).into(),
        params.app_id.into(),
        query_start.clone().into(),
        query_end.clone().into(),
    ];
    let total = state
        .db
        .query_one(Statement::from_sql_and_values(
            state.db.get_database_backend(),
            count_sql,
            common_values.clone(),
        ))
        .await?
        .map(|row| row.try_get::<i64>("", "total"))
        .transpose()?
        .map(i64_to_u64)
        .unwrap_or_default();

    let list_sql = format!(
        r#"
        {classified_orders}
        SELECT
            classified_orders."order_no",
            classified_orders."provider_trade_no",
            classified_orders."first_order_no",
            classified_orders."buyer_order_number",
            classified_orders."app_id",
            apps."name" AS "app_name",
            classified_orders."amount_cents",
            classified_orders."provider",
            classified_orders."pay_type",
            classified_orders."provider_buyer_id",
            classified_orders."buyer_user_id",
            users."username" AS "buyer_username",
            users."email" AS "buyer_user_email",
            classified_orders."buyer_email",
            classified_orders."paid_at",
            classified_orders."created_at"
        FROM classified_orders
        INNER JOIN "apps" ON apps."id" = classified_orders."app_id"
        LEFT JOIN "users" ON users."id" = classified_orders."buyer_user_id"
        WHERE classified_orders."buyer_order_number" > 1
          AND classified_orders."event_time" >= $3::TIMESTAMP
          AND classified_orders."event_time" < $4::TIMESTAMP
        ORDER BY COALESCE(classified_orders."paid_at", classified_orders."created_at") DESC,
                 classified_orders."id" DESC
        LIMIT $5 OFFSET $6
        "#
    );
    let mut list_values = common_values;
    list_values.push(limit.into());
    list_values.push(offset.into());
    let rows = state
        .db
        .query_all(Statement::from_sql_and_values(
            state.db.get_database_backend(),
            list_sql,
            list_values,
        ))
        .await?;
    let list = rows
        .into_iter()
        .map(|row| {
            Ok(DashboardReturningOrder {
                order_no: row.try_get("", "order_no")?,
                provider_trade_no: row.try_get("", "provider_trade_no")?,
                first_order_no: row.try_get("", "first_order_no")?,
                purchase_number: i64_to_u64(row.try_get("", "buyer_order_number")?),
                app_id: row.try_get("", "app_id")?,
                app_name: row.try_get("", "app_name")?,
                amount_cents: row.try_get("", "amount_cents")?,
                provider: row.try_get("", "provider")?,
                pay_type: row.try_get("", "pay_type")?,
                provider_buyer_id: row.try_get("", "provider_buyer_id")?,
                buyer_user_id: row.try_get("", "buyer_user_id")?,
                buyer_username: row.try_get("", "buyer_username")?,
                buyer_user_email: row.try_get("", "buyer_user_email")?,
                buyer_email: row.try_get("", "buyer_email")?,
                paid_at: row.try_get("", "paid_at")?,
                created_at: row.try_get("", "created_at")?,
            })
        })
        .collect::<Result<Vec<_>, sea_orm::DbErr>>()?;

    Ok(PagingResponse { list, page, total })
}

fn trend_periods(
    group_by: TrendGroupBy,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<Vec<String>, AppError> {
    let timezone = business_timezone()?;
    let today = Utc::now().with_timezone(&timezone).date_naive();
    let custom_range = match (start_date, end_date) {
        (None, None) => None,
        (Some(start), Some(end)) => Some((parse_trend_date(start)?, parse_trend_date(end)?)),
        _ => {
            return Err(AppError::validation(
                "start_date and end_date must be provided together",
            ));
        }
    };

    match group_by {
        TrendGroupBy::Hour => {
            let (start, end) = custom_range.unwrap_or((today, today));
            let day_count = (end - start).num_days() + 1;
            if day_count < 1 {
                return Err(AppError::validation(
                    "start_date must not be later than end_date",
                ));
            }
            if day_count > MAX_HOURLY_DAYS {
                return Err(AppError::validation(
                    "hourly range must not exceed 366 days",
                ));
            }
            let start = start
                .and_hms_opt(0, 0, 0)
                .ok_or_else(|| AppError::validation("invalid trend start hour"))?;
            Ok((0..day_count * HOURLY_PERIODS)
                .map(|offset| {
                    (start + Duration::hours(offset))
                        .format("%Y-%m-%d %H:00")
                        .to_string()
                })
                .collect())
        }
        TrendGroupBy::Day => {
            let (start, end) =
                custom_range.unwrap_or_else(|| (today - Duration::days(DAILY_PERIODS - 1), today));
            let period_count = (end - start).num_days() + 1;
            if period_count < 1 {
                return Err(AppError::validation(
                    "start_date must not be later than end_date",
                ));
            }
            if period_count > MAX_DAILY_PERIODS {
                return Err(AppError::validation(
                    "daily range must not exceed 3660 days",
                ));
            }
            Ok((0..period_count)
                .map(|offset| {
                    (start + Duration::days(offset))
                        .format("%Y-%m-%d")
                        .to_string()
                })
                .collect())
        }
        TrendGroupBy::Month => {
            let current_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .ok_or_else(|| AppError::validation("invalid current month"))?;
            let (start, end) = match custom_range {
                Some((start, end)) => (month_start(start)?, month_start(end)?),
                None => (
                    current_month
                        .checked_sub_months(Months::new(MONTHLY_PERIODS - 1))
                        .ok_or_else(|| AppError::validation("invalid trend start month"))?,
                    current_month,
                ),
            };
            let period_count =
                (end.year() - start.year()) * 12 + end.month() as i32 - start.month() as i32 + 1;
            if period_count < 1 {
                return Err(AppError::validation(
                    "start_date must not be later than end_date",
                ));
            }
            if period_count > MAX_MONTHLY_PERIODS {
                return Err(AppError::validation(
                    "monthly range must not exceed 600 months",
                ));
            }
            (0..period_count as u32)
                .map(|offset| {
                    start
                        .checked_add_months(Months::new(offset))
                        .map(|date| date.format("%Y-%m").to_string())
                        .ok_or_else(|| AppError::validation("invalid trend month"))
                })
                .collect()
        }
        TrendGroupBy::Year => {
            let current_year = year_start(today)?;
            let (start, end) = match custom_range {
                Some((start, end)) => (year_start(start)?, year_start(end)?),
                None => (
                    current_year
                        .checked_sub_months(Months::new((YEARLY_PERIODS - 1) * 12))
                        .ok_or_else(|| AppError::validation("invalid trend start year"))?,
                    current_year,
                ),
            };
            let period_count = end.year() - start.year() + 1;
            if period_count < 1 {
                return Err(AppError::validation(
                    "start_date must not be later than end_date",
                ));
            }
            if period_count > MAX_YEARLY_PERIODS {
                return Err(AppError::validation(
                    "yearly range must not exceed 100 years",
                ));
            }
            (0..period_count as u32)
                .map(|offset| {
                    start
                        .checked_add_months(Months::new(offset * 12))
                        .map(|date| date.format("%Y").to_string())
                        .ok_or_else(|| AppError::validation("invalid trend year"))
                })
                .collect()
        }
    }
}

fn parse_trend_date(value: &str) -> Result<NaiveDate, AppError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::validation("dates must use YYYY-MM-DD format"))
}

fn returning_order_period_bounds(
    group_by: TrendGroupBy,
    period: &str,
) -> Result<(String, String), AppError> {
    let start = match group_by {
        TrendGroupBy::Hour => {
            let value =
                chrono::NaiveDateTime::parse_from_str(period, "%Y-%m-%d %H:%M").map_err(|_| {
                    AppError::validation("hour period must use YYYY-MM-DD HH:00 format")
                })?;
            if value.format("%Y-%m-%d %H:00").to_string() != period {
                return Err(AppError::validation(
                    "hour period must use YYYY-MM-DD HH:00 format",
                ));
            }
            value
        }
        TrendGroupBy::Day => parse_trend_date(period)?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::validation("invalid day period"))?,
        TrendGroupBy::Month => NaiveDate::parse_from_str(&format!("{period}-01"), "%Y-%m-%d")
            .map_err(|_| AppError::validation("month period must use YYYY-MM format"))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::validation("invalid month period"))?,
        TrendGroupBy::Year => NaiveDate::parse_from_str(&format!("{period}-01-01"), "%Y-%m-%d")
            .map_err(|_| AppError::validation("year period must use YYYY format"))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::validation("invalid year period"))?,
    };
    let end = match group_by {
        TrendGroupBy::Hour => start.checked_add_signed(Duration::hours(1)),
        TrendGroupBy::Day => start.checked_add_signed(Duration::days(1)),
        TrendGroupBy::Month => start.checked_add_months(Months::new(1)),
        TrendGroupBy::Year => start.checked_add_months(Months::new(12)),
    }
    .ok_or_else(|| AppError::validation("invalid returning-order period"))?;
    Ok((
        start.format("%Y-%m-%d %H:%M:%S").to_string(),
        end.format("%Y-%m-%d %H:%M:%S").to_string(),
    ))
}

fn month_start(date: NaiveDate) -> Result<NaiveDate, AppError> {
    NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
        .ok_or_else(|| AppError::validation("invalid trend month"))
}

fn year_start(date: NaiveDate) -> Result<NaiveDate, AppError> {
    NaiveDate::from_ymd_opt(date.year(), 1, 1)
        .ok_or_else(|| AppError::validation("invalid trend year"))
}

fn business_timezone() -> Result<FixedOffset, AppError> {
    FixedOffset::east_opt(8 * 60 * 60)
        .ok_or_else(|| AppError::validation("invalid business timezone"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDateTime, Timelike};

    #[test]
    fn trend_periods_have_expected_lengths_and_formats() {
        let hours = trend_periods(TrendGroupBy::Hour, None, None).unwrap();
        let days = trend_periods(TrendGroupBy::Day, None, None).unwrap();
        let months = trend_periods(TrendGroupBy::Month, None, None).unwrap();
        let years = trend_periods(TrendGroupBy::Year, None, None).unwrap();

        assert_eq!(hours.len(), HOURLY_PERIODS as usize);
        assert_eq!(days.len(), DAILY_PERIODS as usize);
        assert_eq!(months.len(), MONTHLY_PERIODS as usize);
        assert_eq!(years.len(), YEARLY_PERIODS as usize);
        let first_hour = NaiveDateTime::parse_from_str(&hours[0], "%Y-%m-%d %H:%M").unwrap();
        assert_eq!(first_hour.minute(), 0);
        assert!(NaiveDate::parse_from_str(&days[0], "%Y-%m-%d").is_ok());
        assert_eq!(months[0].len(), 7);
        assert_eq!(years[0].len(), 4);
    }

    #[test]
    fn trend_periods_use_custom_date_ranges() {
        let hours =
            trend_periods(TrendGroupBy::Hour, Some("2026-08-01"), Some("2026-08-02")).unwrap();
        let days =
            trend_periods(TrendGroupBy::Day, Some("2026-08-01"), Some("2026-08-03")).unwrap();
        let months =
            trend_periods(TrendGroupBy::Month, Some("2026-01-15"), Some("2026-03-31")).unwrap();
        let years =
            trend_periods(TrendGroupBy::Year, Some("2024-08-01"), Some("2026-02-01")).unwrap();

        assert_eq!(hours.len(), 48);
        assert_eq!(hours.first().map(String::as_str), Some("2026-08-01 00:00"));
        assert_eq!(hours.last().map(String::as_str), Some("2026-08-02 23:00"));
        assert_eq!(
            trend_periods(TrendGroupBy::Hour, Some("2026-01-01"), Some("2027-01-01"))
                .unwrap()
                .len(),
            366 * HOURLY_PERIODS as usize
        );
        assert_eq!(days, ["2026-08-01", "2026-08-02", "2026-08-03"]);
        assert_eq!(months, ["2026-01", "2026-02", "2026-03"]);
        assert_eq!(years, ["2024", "2025", "2026"]);
        assert!(trend_periods(TrendGroupBy::Day, Some("2026-08-01"), None).is_err());
        assert!(trend_periods(TrendGroupBy::Day, Some("2026-08-03"), Some("2026-08-01")).is_err());
        assert!(trend_periods(TrendGroupBy::Hour, Some("2026-08-01"), Some("2027-08-02")).is_err());
    }

    #[test]
    fn trend_group_by_rejects_unknown_values() {
        assert!(matches!(
            TrendGroupBy::parse(Some("hour")),
            Ok(TrendGroupBy::Hour)
        ));
        assert!(matches!(
            TrendGroupBy::parse(Some("year")),
            Ok(TrendGroupBy::Year)
        ));
        assert!(TrendGroupBy::parse(Some("week")).is_err());
    }

    #[test]
    fn returning_order_period_bounds_match_chart_periods() {
        assert_eq!(
            returning_order_period_bounds(TrendGroupBy::Hour, "2026-09-10 23:00").unwrap(),
            (
                "2026-09-10 23:00:00".to_string(),
                "2026-09-11 00:00:00".to_string()
            )
        );
        assert_eq!(
            returning_order_period_bounds(TrendGroupBy::Day, "2026-09-10").unwrap(),
            (
                "2026-09-10 00:00:00".to_string(),
                "2026-09-11 00:00:00".to_string()
            )
        );
        assert_eq!(
            returning_order_period_bounds(TrendGroupBy::Month, "2026-12").unwrap(),
            (
                "2026-12-01 00:00:00".to_string(),
                "2027-01-01 00:00:00".to_string()
            )
        );
        assert_eq!(
            returning_order_period_bounds(TrendGroupBy::Year, "2026").unwrap(),
            (
                "2026-01-01 00:00:00".to_string(),
                "2027-01-01 00:00:00".to_string()
            )
        );
        assert!(returning_order_period_bounds(TrendGroupBy::Day, "2026-02-30").is_err());
        assert!(returning_order_period_bounds(TrendGroupBy::Month, "2026-13").is_err());
    }
}
