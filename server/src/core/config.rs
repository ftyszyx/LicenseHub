use crate::core::my_error::AppError;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
    pub payment: PaymentConfig,
    pub register_open: bool,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_host: String,
    pub db_port: u16,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub db_url: String,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub port: u16,
    pub host: String,
    pub username: String,
    pub password: String,
    pub key_prefix: String,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expire_days: u32,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub enabled: bool,
    pub pay_types: Vec<String>,
    pub public_base_url: String,
    pub frontend_base_url: String,
    pub site_name: String,
    pub wechat_native: WechatNativePayConfig,
}

#[derive(Debug, Clone)]
pub struct WechatNativePayConfig {
    pub enabled: bool,
    pub app_id: String,
    pub mch_id: String,
    pub merchant_serial_no: String,
    pub merchant_private_key_pem: String,
    pub api_v3_key: String,
    pub platform_public_key_pem: Option<String>,
    pub api_base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Config {
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            jwt: JwtConfig::from_env()?,
            server: ServerConfig::from_env()?,
            payment: PaymentConfig::from_env()?,
            register_open: env::var("REGISTER_OPEN")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .map_err(|_| AppError::Message("Invalid REGISTER_OPEN value".to_string()))?,
        })
    }
}

impl DatabaseConfig {
    fn from_env() -> Result<Self, AppError> {
        Ok(DatabaseConfig {
            db_name: env::var("DATABASE_NAME")
                .map_err(|_| AppError::Message("DATABASE_NAME must be set".to_string()))?,
            db_user: env::var("DATABASE_USER")
                .map_err(|_| AppError::Message("DATABASE_USER must be set".to_string()))?,
            db_password: env::var("DATABASE_PASSWORD")
                .map_err(|_| AppError::Message("DATABASE_PASSWORD must be set".to_string()))?,
            db_host: env::var("DATABASE_HOST")
                .map_err(|_| AppError::Message("DATABASE_HOST must be set".to_string()))?,
            db_port: env::var("DATABASE_PORT")
                .map_err(|_| AppError::Message("DATABASE_PORT must be set".to_string()))?
                .parse()
                .map_err(|_| AppError::Message("Invalid DATABASE_PORT value".to_string()))?,
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| {
                    AppError::Message("Invalid DATABASE_MAX_CONNECTIONS value".to_string())
                })?,
            min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .map_err(|_| {
                    AppError::Message("Invalid DATABASE_MIN_CONNECTIONS value".to_string())
                })?,
            connect_timeout_secs: env::var("DATABASE_CONNECT_TIMEOUT")
                .unwrap_or_else(|_| "8".to_string())
                .parse()
                .map_err(|_| AppError::Message("Invalid DB_CONNECT_TIMEOUT value".to_string()))?,
            db_url: format!(
                "postgres://{}:{}@{}:{}/{}",
                env::var("DATABASE_USER").unwrap(),
                env::var("DATABASE_PASSWORD").unwrap(),
                env::var("DATABASE_HOST").unwrap(),
                env::var("DATABASE_PORT").unwrap(),
                env::var("DATABASE_NAME").unwrap()
            ),
        })
    }
}

impl RedisConfig {
    fn from_env() -> Result<Self, AppError> {
        Ok(RedisConfig {
            port: env::var("REDIS_PORT")
                .map_err(|_| AppError::Message("REDIS_PORT must be set".to_string()))?
                .parse()
                .map_err(|_| AppError::Message("Invalid REDIS_PORT value".to_string()))?,
            host: env::var("REDIS_HOST")
                .map_err(|_| AppError::Message("REDIS_HOST must be set".to_string()))?,
            username: env::var("REDIS_USERNAME")
                .map_err(|_| AppError::Message("REDIS_USERNAME must be set".to_string()))?,
            password: env::var("REDIS_PASSWORD")
                .map_err(|_| AppError::Message("REDIS_PASSWORD must be set".to_string()))?,
            key_prefix: env::var("REDIS_KEY_PREFIX").unwrap_or_else(|_| "hub_".to_string()),
        })
    }
}

impl JwtConfig {
    fn from_env() -> Result<Self, AppError> {
        Ok(JwtConfig {
            secret: env::var("JWT_SECRET")
                .map_err(|_| AppError::Message("JWT_SECRET must be set".to_string()))?,
            expire_days: env::var("JWT_EXPIRE")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .map_err(|_| AppError::Message("Invalid JWT_EXPIRE value".to_string()))?,
        })
    }
}

impl ServerConfig {
    fn from_env() -> Result<Self, AppError> {
        Ok(ServerConfig {
            host: env::var("LISTEN_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("LISTEN_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| AppError::Message("Invalid LISTEN_PORT value".to_string()))?,
        })
    }
}

impl PaymentConfig {
    fn from_env() -> Result<Self, AppError> {
        let enabled = env::var("PAYMENT_ENABLED")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .map_err(|_| AppError::Message("Invalid PAYMENT_ENABLED value".to_string()))?;
        Ok(PaymentConfig {
            enabled,
            pay_types: env::var("PAYMENT_PAY_TYPES")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
            public_base_url: env::var("PUBLIC_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .trim_end_matches('/')
                .to_string(),
            frontend_base_url: env::var("FRONTEND_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:9070".to_string())
                .trim_end_matches('/')
                .to_string(),
            site_name: env::var("PAYMENT_SITE_NAME").unwrap_or_else(|_| "LicenseHub".to_string()),
            wechat_native: WechatNativePayConfig::from_env()?,
        })
    }
}

impl WechatNativePayConfig {
    fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            enabled: env::var("WECHAT_PAY_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .map_err(|_| AppError::Message("Invalid WECHAT_PAY_ENABLED value".to_string()))?,
            app_id: env::var("WECHAT_PAY_APP_ID").unwrap_or_default(),
            mch_id: env::var("WECHAT_PAY_MCH_ID").unwrap_or_default(),
            merchant_serial_no: env::var("WECHAT_PAY_MERCHANT_SERIAL_NO").unwrap_or_default(),
            merchant_private_key_pem: read_env_or_file(
                "WECHAT_PAY_MERCHANT_PRIVATE_KEY",
                "WECHAT_PAY_MERCHANT_PRIVATE_KEY_PATH",
            )?
            .unwrap_or_default(),
            api_v3_key: env::var("WECHAT_PAY_API_V3_KEY").unwrap_or_default(),
            platform_public_key_pem: match read_env_or_file(
                "WECHAT_PAY_PLATFORM_PUBLIC_KEY",
                "WECHAT_PAY_PLATFORM_PUBLIC_KEY_PATH",
            )? {
                Some(value) => Some(value),
                None => {
                    read_env_or_file("WECHAT_PAY_PLATFORM_CERT", "WECHAT_PAY_PLATFORM_CERT_PATH")?
                }
            },
            api_base_url: env::var("WECHAT_PAY_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.mch.weixin.qq.com".to_string())
                .trim_end_matches('/')
                .to_string(),
        })
    }
}

fn read_env_or_file(value_key: &str, path_key: &str) -> Result<Option<String>, AppError> {
    if let Ok(value) = env::var(value_key) {
        if !value.trim().is_empty() {
            return Ok(Some(value));
        }
    }
    if let Ok(path) = env::var(path_key) {
        let path = path.trim();
        if !path.is_empty() {
            return std::fs::read_to_string(path).map(Some).map_err(|error| {
                AppError::Message(format!("failed to read {}: {}", path_key, error))
            });
        }
    }
    Ok(None)
}
