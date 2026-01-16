use crate::core::my_error::AppError;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub server: ServerConfig,
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

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Config {
            database: DatabaseConfig::from_env()?,
            redis: RedisConfig::from_env()?,
            jwt: JwtConfig::from_env()?,
            server: ServerConfig::from_env()?,
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
