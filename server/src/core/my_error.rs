use crate::core::constants::*;
use crate::core::response::ApiResponse;
use salvo::{
    http::{ParseError, StatusCode},
    prelude::*,
};
use sea_orm::DbErr;
use std::{convert::Infallible, fmt};

/// 改进的错误处理系统
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    /// 数据库相关错误
    Database(DbErr),
    /// 资源未找到错误
    NotFound {
        resource: String,
        id: Option<i32>,
    },
    /// 验证错误
    Validation {
        message: String,
    },
    /// 认证失败
    AuthFailed {
        reason: String,
    },
    /// 权限不足
    Forbidden {
        action: String,
    },
    NotImplemented {
        message: String,
    },
    /// 业务逻辑错误
    BusinessLogic {
        code: String,
        message: String,
    },
    /// 外部服务错误
    ExternalService {
        service: String,
        error: String,
    },
    UserAlreadyExists,
    InternalError {
        message: String,
    },
    UserNotFound {
        message: String,
    },
    Message(String),
}

#[allow(dead_code)]
impl AppError {
    /// 创建验证错误的便捷方法
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// 创建未找到错误的便捷方法
    pub fn not_found(resource: impl Into<String>, id: Option<i32>) -> Self {
        Self::NotFound {
            resource: resource.into(),
            id,
        }
    }

    /// 创建认证错误的便捷方法
    pub fn auth_failed(reason: impl Into<String>) -> Self {
        Self::AuthFailed {
            reason: reason.into(),
        }
    }

    pub fn user_already_exists() -> Self {
        Self::UserAlreadyExists
    }

    /// 创建业务逻辑错误的便捷方法
    pub fn business_logic(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BusinessLogic {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 获取错误代码
    pub fn error_code(&self) -> u16 {
        match self {
            Self::Database(_) => APP_OTHER,
            Self::NotFound { .. } => APP_NOT_FOUND,
            Self::Validation { .. } => APP_VALIDATION_ERROR,
            Self::AuthFailed { .. } => APP_AUTH_FAILED,
            Self::Forbidden { .. } => APP_FORBIDDEN,
            Self::BusinessLogic { .. } => APP_BUSINESS_LOGIC,
            Self::ExternalService { .. } => APP_EXTERNAL_SERVICE,
            Self::UserAlreadyExists => APP_USER_ALREADY_EXISTS,
            Self::NotImplemented { .. } => APP_NOT_IMPLEMENTED,
            Self::Message(_) => APP_OTHER,
            Self::InternalError { .. } => APP_INTERNAL_ERROR,
            Self::UserNotFound { .. } => APP_USER_NOT_FOUND,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::NotFound { resource, id } => match id {
                Some(id) => write!(f, "Resource '{}' with id {} not found", resource, id),
                None => write!(f, "Resource '{}' not found", resource),
            },
            Self::Validation { message } => {
                write!(f, "Validation error: {}", message)
            }
            Self::AuthFailed { reason } => write!(f, "Authentication failed: {}", reason),
            Self::Forbidden { action } => write!(f, "Forbidden action: {}", action),
            Self::BusinessLogic { code, message } => {
                write!(f, "Business logic error [{}]: {}", code, message)
            }
            Self::ExternalService { service, error } => {
                write!(f, "External service '{}' error: {}", service, error)
            }
            Self::UserAlreadyExists => write!(f, "User already exists"),
            Self::NotImplemented { message } => write!(f, "Not implemented: {}", message),
            Self::Message(message) => write!(f, "{}", message),
            Self::InternalError { message } => write!(f, "Internal error: {}", message),
            Self::UserNotFound { message } => write!(f, "User not found: {}", message),
        }
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        tracing::error!("Validation errors: {:?}", errors);
        Self::Validation {
            message: errors.to_string(),
        }
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        tracing::error!("Database error: {:?}", err);
        Self::Database(err)
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        tracing::error!("Password hashing error: {}", err);
        Self::BusinessLogic {
            code: "PASSWORD_HASH_ERROR".to_string(),
            message: "Failed to hash password".to_string(),
        }
    }
}

impl From<Infallible> for AppError {
    fn from(err: Infallible) -> Self {
        tracing::error!("Infallible error: {:?}", err);
        Self::Message("Infallible error".to_string())
    }
}

impl From<ParseError> for AppError {
    fn from(err: ParseError) -> Self {
        tracing::error!("Parse error: {:?}", err);
        Self::Message("Parse error".to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Anyhow error: {:?}", err);
        Self::Message(err.to_string())
    }
}

#[salvo::async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        tracing::error!("AppError: {:?}", self.to_string());
        res.status_code = Some(StatusCode::OK);
        res.render(Json(ApiResponse::<String>::error_with_message_and_code(
            self.to_string(),
            self.error_code(),
        )));
    }
}

impl salvo_oapi::EndpointOutRegister for AppError {
    fn register(components: &mut salvo_oapi::Components, operation: &mut salvo_oapi::Operation) {
        operation
            .responses
            .append(&mut salvo::prelude::StatusError::to_responses(components));
    }
}
