use crate::core::my_error::AppError;
use redis::{AsyncCommands, Client, RedisError, aio::MultiplexedConnection};
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Clone)]
pub struct RedisCache {
    client: Client,
}

#[allow(dead_code)]
impl RedisCache {
    /// 创建一个新的 RedisCache 实例
    pub fn new(redis_url: &str, username: &str, password: &str) -> Result<Self, RedisError> {
        let url_with_auth = if username.is_empty() && password.is_empty() {
            redis_url.to_string()
        } else {
            let (scheme, rest) = redis_url.split_once("://").ok_or_else(|| {
                RedisError::from((redis::ErrorKind::InvalidClientConfig, "invalid redis url"))
            })?;

            if rest.contains('@') {
                redis_url.to_string()
            } else {
                let auth = if username.is_empty() {
                    format!(":{}@", password)
                } else {
                    format!("{}:{}@", username, password)
                };
                format!("{}://{}{}", scheme, auth, rest)
            }
        };

        let client = Client::open(url_with_auth)?;
        Ok(Self { client })
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection, RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// 值会从 JSON 字符串反序列化
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
        let mut conn = self.get_conn().await?;
        let result: Option<String> = conn.get(key).await?;
        match result {
            Some(val_str) => {
                let val: T = serde_json::from_str(&val_str)?;
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Result<(), AppError> {
        let mut conn = self.get_conn().await?;
        let val_str = serde_json::to_string(value)?;
        if let Some(duration) = ttl {
            let _: () = conn.set_ex(key, val_str, duration.as_secs() as u64).await?;
        } else {
            let _: () = conn.set(key, val_str).await?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn del(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.get_conn().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }
}

// 将 Redis 错误转换为自定义的 AppError
impl From<RedisError> for AppError {
    fn from(err: RedisError) -> Self {
        tracing::error!("Redis error: {:?}", err);
        AppError::ExternalService {
            service: "Redis".to_string(),
            error: err.to_string(),
        }
    }
}

// 将 serde_json 错误转换为自定义的 AppError
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("Serialization/Deserialization error: {:?}", err);
        AppError::BusinessLogic {
            code: "SERIALIZATION_ERROR".to_string(),
            message: err.to_string(),
        }
    }
}
