use crate::core::config::*;
use crate::core::my_error::*;
use crate::core::redis_cache::RedisCache;
use chrono::{FixedOffset, Utc};
use payment_adapter::{PaymentRegistry, WechatNativeAdapter, WechatNativeConfig};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::sync::Arc;
use std::time::Duration;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{fmt::time::FormatTime, layer::SubscriberExt, util::SubscriberInitExt};

struct East8Timer;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: Arc<RedisCache>,
    pub config: Arc<Config>,
    pub payment_registry: Arc<PaymentRegistry>,
}

impl FormatTime for East8Timer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let east8 = FixedOffset::east_opt(8 * 3600).unwrap();
        let now = Utc::now().with_timezone(&east8);
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f"))
    }
}

pub async fn init_app() -> Result<AppState, AppError> {
    // 加载配置
    let config = Config::from_env()
        .map_err(|e| AppError::Message(format!("config load failed:{}", e.to_string())))?;
    tracing::info!("Configuration loaded successfully");
    // 初始化数据库
    let db_pool = init_db(&config.database)
        .await
        .map_err(|e| AppError::Message(format!("database connection failed:{}", e)))?;
    tracing::info!("Database connected successfully");
    // 初始化 Redis
    let redis_url = format!("redis://{}:{}", config.redis.host, config.redis.port);
    let redis = RedisCache::new(
        &redis_url,
        &config.redis.username,
        &config.redis.password,
        &config.redis.key_prefix,
    )
    .map_err(|e| AppError::Message(format!("redis connection failed:{}", e)))?;
    tracing::info!("Redis connected successfully");
    let payment_registry = init_payment_registry(&config)?;
    // 创建应用状态
    let app_state = AppState {
        db: db_pool,
        redis: Arc::new(redis),
        config: Arc::new(config),
        payment_registry: Arc::new(payment_registry),
    };
    // 创建路由
    Ok(app_state)
}

fn init_payment_registry(config: &Config) -> Result<PaymentRegistry, AppError> {
    let mut registry = PaymentRegistry::new();
    if !config.payment.enabled {
        return Ok(registry);
    }

    let enabled_types = &config.payment.pay_types;
    let wechat_enabled = config.payment.wechat_native.enabled
        && (enabled_types.is_empty()
            || enabled_types
                .iter()
                .any(|pay_type| pay_type == "wechat_native"));
    if wechat_enabled {
        let cfg = &config.payment.wechat_native;
        let adapter = WechatNativeAdapter::new(WechatNativeConfig {
            app_id: cfg.app_id.clone(),
            mch_id: cfg.mch_id.clone(),
            merchant_serial_no: cfg.merchant_serial_no.clone(),
            merchant_private_key_pem: cfg.merchant_private_key_pem.clone(),
            api_v3_key: cfg.api_v3_key.clone(),
            platform_public_key_pem: cfg.platform_public_key_pem.clone(),
            api_base_url: cfg.api_base_url.clone(),
        })
        .map_err(|error| {
            AppError::Message(format!("failed to initialize WeChat Native pay: {}", error))
        })?;
        registry.register(adapter);
    }

    Ok(registry)
}

pub async fn init_db(config: &DatabaseConfig) -> Result<DatabaseConnection, DbErr> {
    tracing::info!("Connecting to database: {}", config.db_url);
    let mut opt = ConnectOptions::new(&config.db_url);
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
        .sqlx_logging(true);
    let db = Database::connect(opt).await?;
    Ok(db)
}

pub fn init_log() -> WorkerGuard {
    // 同时输出到文件和 stdout，并保留 guard 确保文件日志 flush
    let file_appender = rolling::daily("logs", "app.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);
    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let fmt_file = tracing_subscriber::fmt::layer()
        .with_timer(East8Timer)
        .with_ansi(false)
        .with_writer(non_blocking_appender);
    let fmt_stdout = tracing_subscriber::fmt::layer().with_timer(East8Timer);
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_file)
        .with(fmt_stdout)
        .try_init();
    guard
}
