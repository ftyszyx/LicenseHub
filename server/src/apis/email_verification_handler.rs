use crate::apis::system_settings_handler::{get_email_service_config, get_registration_enabled};
use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use crate::mailer::send_email_code;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use data_model::{email_verification_challenges, email_verification_tokens, users};
use hmac::{Hmac, Mac};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration as StdDuration;
use uuid::Uuid;

const PURPOSE_REGISTER: &str = "register";
const EMAIL_CODE_TTL_MINUTES: i64 = 10;
const EMAIL_CODE_RESEND_SECONDS: i64 = 60;
const EMAIL_CODE_MAX_ATTEMPTS: i32 = 5;
const REGISTRATION_TOKEN_TTL_MINUTES: i64 = 15;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Deserialize)]
pub struct StartEmailVerificationReq {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct StartEmailVerificationInfo {
    pub challenge_id: String,
    pub expires_in_seconds: i64,
    pub resend_after_seconds: i64,
}

#[derive(Debug, Deserialize)]
pub struct VerifyEmailCodeReq {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailCodeInfo {
    pub verification_token: String,
    pub expires_in_seconds: i64,
}

#[handler]
pub async fn start_email_verification(
    depot: &mut Depot,
    req: &mut Request,
    body: JsonBody<StartEmailVerificationReq>,
) -> Result<ApiResponse<StartEmailVerificationInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    ensure_registration_ready(state).await?;
    let secret = email_secret(state)?;
    let email = normalize_email(&body.email)?;
    if users::Entity::find()
        .filter(users::Column::Email.eq(&email))
        .one(&state.db)
        .await?
        .is_some()
    {
        return Err(AppError::business_logic(
            "EMAIL_ALREADY_REGISTERED",
            "该邮箱已注册",
        ));
    }

    let client_ip = client_ip(req);
    let now = Utc::now().fixed_offset();
    if let Some(recent) = email_verification_challenges::Entity::find()
        .filter(email_verification_challenges::Column::Email.eq(&email))
        .filter(email_verification_challenges::Column::Purpose.eq(PURPOSE_REGISTER))
        .filter(email_verification_challenges::Column::ResendAfter.gt(now))
        .order_by_desc(email_verification_challenges::Column::CreatedAt)
        .one(&state.db)
        .await?
    {
        let retry = (recent.resend_after - now).num_seconds().max(1);
        return Err(AppError::business_logic(
            "EMAIL_CODE_RATE_LIMITED",
            format!("发送过于频繁，请在 {retry} 秒后重试"),
        ));
    }
    enforce_email_rate_limits(state, &email, &client_ip).await?;

    let challenge_id = Uuid::new_v4();
    let code = random_digits(6)?;
    let code_hash = email_code_hash(secret, challenge_id, &email, &code)?;
    let challenge = email_verification_challenges::ActiveModel {
        id: Set(challenge_id),
        email: Set(email.clone()),
        purpose: Set(PURPOSE_REGISTER.to_string()),
        code_hash: Set(code_hash),
        attempts: Set(0),
        expires_at: Set(now + Duration::minutes(EMAIL_CODE_TTL_MINUTES)),
        resend_after: Set(now + Duration::seconds(EMAIL_CODE_RESEND_SECONDS)),
        sent_at: Set(None),
        send_failed_at: Set(None),
        verified_at: Set(None),
        consumed_at: Set(None),
        created_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    let config = get_email_service_config(state).await?;
    match send_email_code(&config, &email, &code, EMAIL_CODE_TTL_MINUTES).await {
        Ok(()) => {
            let mut active = challenge.into_active_model();
            active.sent_at = Set(Some(Utc::now().fixed_offset()));
            active.update(&state.db).await?;
        }
        Err(error) => {
            let mut active = challenge.into_active_model();
            active.send_failed_at = Set(Some(Utc::now().fixed_offset()));
            active.update(&state.db).await?;
            return Err(error);
        }
    }

    Ok(ApiResponse::success(StartEmailVerificationInfo {
        challenge_id: challenge_id.to_string(),
        expires_in_seconds: EMAIL_CODE_TTL_MINUTES * 60,
        resend_after_seconds: EMAIL_CODE_RESEND_SECONDS,
    }))
}

#[handler]
pub async fn verify_email_code(
    depot: &mut Depot,
    challenge_id: PathParam<String>,
    body: JsonBody<VerifyEmailCodeReq>,
) -> Result<ApiResponse<VerifyEmailCodeInfo>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    ensure_registration_ready(state).await?;
    let secret = email_secret(state)?;
    let challenge_id = Uuid::parse_str(&challenge_id.into_inner())
        .map_err(|_| AppError::business_logic("EMAIL_CODE_INVALID", "邮箱验证码无效"))?;
    let code = normalize_email_code(&body.code)?;
    let tx = state.db.begin().await?;
    let challenge = email_verification_challenges::Entity::find_by_id(challenge_id)
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::business_logic("EMAIL_CODE_INVALID", "邮箱验证码无效"))?;
    let now = Utc::now().fixed_offset();
    if challenge.consumed_at.is_some() || challenge.verified_at.is_some() {
        return Err(AppError::business_logic(
            "EMAIL_CODE_INVALID",
            "邮箱验证码已使用",
        ));
    }
    if challenge.expires_at <= now {
        return Err(AppError::business_logic(
            "EMAIL_CODE_EXPIRED",
            "邮箱验证码已过期",
        ));
    }
    if challenge.attempts >= EMAIL_CODE_MAX_ATTEMPTS {
        return Err(AppError::business_logic(
            "EMAIL_CODE_ATTEMPTS_EXCEEDED",
            "邮箱验证码错误次数过多",
        ));
    }
    let expected = email_code_hash(secret, challenge_id, &challenge.email, &code)?;
    if !constant_time_eq(&expected, &challenge.code_hash) {
        let next_attempts = challenge.attempts + 1;
        let mut active = challenge.into_active_model();
        active.attempts = Set(next_attempts);
        active.update(&tx).await?;
        tx.commit().await?;
        let (code, message) = if next_attempts >= EMAIL_CODE_MAX_ATTEMPTS {
            ("EMAIL_CODE_ATTEMPTS_EXCEEDED", "邮箱验证码错误次数过多")
        } else {
            ("EMAIL_CODE_INVALID", "邮箱验证码错误")
        };
        return Err(AppError::business_logic(code, message));
    }

    let email = challenge.email.clone();
    let mut active = challenge.into_active_model();
    active.verified_at = Set(Some(now));
    active.update(&tx).await?;
    let token = random_token()?;
    let token_hash = token_hash(&token);
    email_verification_tokens::ActiveModel {
        token_hash: Set(token_hash),
        challenge_id: Set(challenge_id),
        email: Set(email),
        purpose: Set(PURPOSE_REGISTER.to_string()),
        expires_at: Set(now + Duration::minutes(REGISTRATION_TOKEN_TTL_MINUTES)),
        consumed_at: Set(None),
        created_at: Set(now),
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;

    Ok(ApiResponse::success(VerifyEmailCodeInfo {
        verification_token: token,
        expires_in_seconds: REGISTRATION_TOKEN_TTL_MINUTES * 60,
    }))
}

pub fn normalize_email(value: &str) -> Result<String, AppError> {
    let email = value.trim().to_ascii_lowercase();
    if email.len() > 320 || email.parse::<email_address::EmailAddress>().is_err() {
        return Err(AppError::business_logic("EMAIL_INVALID", "邮箱格式不正确"));
    }
    Ok(email)
}

pub fn token_hash(token: &str) -> String {
    hex_digest(Sha256::digest(token.as_bytes()).as_slice())
}

async fn ensure_registration_ready(state: &AppState) -> Result<(), AppError> {
    if !get_registration_enabled(state).await? {
        return Err(AppError::business_logic(
            "REGISTRATION_DISABLED",
            "注册功能未开放",
        ));
    }
    email_secret(state)?;
    Ok(())
}

fn email_secret(state: &AppState) -> Result<&str, AppError> {
    state.config.email_code_secret.as_deref().ok_or_else(|| {
        AppError::business_logic("EMAIL_CODE_SECRET_NOT_CONFIGURED", "邮件验证码密钥未配置")
    })
}

fn client_ip(req: &Request) -> String {
    req.remote_addr()
        .as_ipv4()
        .map(|addr| addr.ip().to_string())
        .or_else(|| {
            req.remote_addr()
                .as_ipv6()
                .map(|addr| addr.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

async fn enforce_email_rate_limits(
    state: &AppState,
    email: &str,
    ip: &str,
) -> Result<(), AppError> {
    let email_key = token_hash(email);
    let ip_key = token_hash(ip);
    let limits = [
        (
            format!("auth:email-cooldown:{email_key}"),
            EMAIL_CODE_RESEND_SECONDS as u64,
            1,
        ),
        (format!("auth:email-rate:email:1h:{email_key}"), 3600, 5),
        (format!("auth:email-rate:email:24h:{email_key}"), 86400, 10),
        (format!("auth:email-rate:ip:1h:{ip_key}"), 3600, 20),
        (format!("auth:email-rate:ip:24h:{ip_key}"), 86400, 50),
    ];
    for (key, ttl, max) in limits {
        let count = state
            .redis
            .increment_counter(&key, StdDuration::from_secs(ttl))
            .await?;
        if count > max {
            return Err(AppError::business_logic(
                "EMAIL_CODE_RATE_LIMITED",
                "验证码发送次数过多，请稍后重试",
            ));
        }
    }
    Ok(())
}

fn normalize_email_code(value: &str) -> Result<String, AppError> {
    let code = value.trim().to_string();
    if code.len() != 6 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(AppError::business_logic(
            "EMAIL_CODE_INVALID",
            "邮箱验证码错误",
        ));
    }
    Ok(code)
}

fn email_code_hash(
    secret: &str,
    challenge_id: Uuid,
    email: &str,
    code: &str,
) -> Result<String, AppError> {
    hmac_hex(
        secret,
        format!("{challenge_id}:{email}:{PURPOSE_REGISTER}:{code}").as_bytes(),
    )
}

fn hmac_hex(secret: &str, value: &[u8]) -> Result<String, AppError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|_| AppError::business_logic("EMAIL_CODE_SECRET_INVALID", "邮件验证码密钥无效"))?;
    mac.update(value);
    Ok(hex_digest(&mac.finalize().into_bytes()))
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

fn random_digits(length: usize) -> Result<String, AppError> {
    let mut bytes = vec![0u8; length];
    getrandom::fill(&mut bytes).map_err(|error| AppError::InternalError {
        message: error.to_string(),
    })?;
    Ok(bytes
        .into_iter()
        .map(|byte| char::from(b'2' + (byte % 8)))
        .collect())
}

fn random_token() -> Result<String, AppError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|error| AppError::InternalError {
        message: error.to_string(),
    })?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_is_normalized_and_validated() {
        assert_eq!(
            normalize_email(" User@Example.COM ").unwrap(),
            "user@example.com"
        );
        assert!(normalize_email("not-an-email").is_err());
    }
}
