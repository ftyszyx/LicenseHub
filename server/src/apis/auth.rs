use crate::apis::auth_middleware::Claims;
use crate::apis::email_verification_handler::{normalize_email, token_hash};
use crate::apis::system_settings_handler::{get_distribution_settings, get_registration_enabled};
use crate::core::app::AppState;
use crate::core::constants;
use crate::core::my_error::AppError;
use crate::core::rbac::list_permission_keys_for_roles;
use crate::core::response::ApiResponse;
use crate::utils::jwt::create_jwt;
use bcrypt::verify;
use chrono::Utc;
use data_model::{
    email_verification_challenges, email_verification_tokens, roles, user_roles, users,
};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::ToSchema;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, EntityTrait, IntoActiveModel,
    QueryFilter, QuerySelect, Set, Statement, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, ToSchema)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterPayload {
    pub username: String,
    pub email: String,
    pub password: String,
    pub verification_token: String,
    #[serde(default)]
    pub referral_code: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct ChangePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

#[handler]
pub async fn register(
    json: JsonBody<RegisterPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    if !get_registration_enabled(state).await? {
        return Err(AppError::business_logic(
            "REGISTRATION_DISABLED",
            "注册功能未开放",
        ));
    }
    let username = normalize_username(&json.username)?;
    validate_password(&json.password)?;
    let email = normalize_email(&json.email)?;
    let distribution = get_distribution_settings(state).await?;
    let requested_referral_code = (distribution.enabled && distribution.referrer_binding_enabled)
        .then(|| {
            json.referral_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty() && value.len() <= 32)
                .map(str::to_ascii_uppercase)
        })
        .flatten();
    let verification_token_hash = token_hash(json.verification_token.trim());
    let password = bcrypt::hash(json.password.clone(), 10)?;
    let tx = state.db.begin().await?;
    let verification = email_verification_tokens::Entity::find_by_id(verification_token_hash)
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or_else(|| {
            AppError::business_logic(
                "EMAIL_VERIFICATION_TOKEN_INVALID",
                "邮箱验证凭证无效或已使用",
            )
        })?;
    let now = Utc::now().fixed_offset();
    if verification.consumed_at.is_some() || verification.purpose != "register" {
        return Err(AppError::business_logic(
            "EMAIL_VERIFICATION_TOKEN_INVALID",
            "邮箱验证凭证无效或已使用",
        ));
    }
    if verification.expires_at <= now {
        return Err(AppError::business_logic(
            "EMAIL_VERIFICATION_TOKEN_EXPIRED",
            "邮箱验证凭证已过期",
        ));
    }
    if verification.email != email {
        return Err(AppError::business_logic(
            "EMAIL_VERIFICATION_TOKEN_INVALID",
            "邮箱与验证凭证不匹配",
        ));
    }
    let user_exists = users::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(users::Column::Username.eq(&username))
                .add(users::Column::Email.eq(&email)),
        )
        .one(&tx)
        .await?;
    if user_exists.is_some() {
        return Err(AppError::user_already_exists());
    }
    let referrer = if let Some(referral_code) = requested_referral_code {
        users::Entity::find()
            .filter(users::Column::ReferralCode.eq(referral_code))
            .lock_shared()
            .one(&tx)
            .await?
    } else {
        None
    };
    let user_role = roles::Entity::find()
        .filter(roles::Column::Name.eq(constants::USER_ROLE))
        .one(&tx)
        .await?;
    let user_role = user_role.ok_or(AppError::not_found("role", None))?;

    let new_user = users::ActiveModel {
        username: Set(username),
        password: Set(password),
        email: Set(Some(email.clone())),
        email_verified_at: Set(Some(now)),
        referral_code: Set(crate::apis::user_handler::new_referral_code()),
        referrer_user_id: Set(referrer.as_ref().map(|user| user.id)),
        referrer_bound_at: Set(referrer.as_ref().map(|_| now)),
        registered_referral_code: Set(referrer.as_ref().map(|user| user.referral_code.clone())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&tx)
    .await?;

    user_roles::ActiveModel {
        user_id: Set(new_user.id),
        role_id: Set(user_role.id),
    }
    .insert(&tx)
    .await?;

    let challenge_id = verification.challenge_id;
    let mut verification_active = verification.into_active_model();
    verification_active.consumed_at = Set(Some(now));
    verification_active.update(&tx).await?;
    email_verification_challenges::Entity::update_many()
        .col_expr(
            email_verification_challenges::Column::ConsumedAt,
            sea_orm::sea_query::Expr::value(Some(now)),
        )
        .filter(email_verification_challenges::Column::Id.eq(challenge_id))
        .exec(&tx)
        .await?;
    tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"UPDATE "orders"
           SET "buyer_user_id" = $1, "updated_at" = $2
           WHERE "buyer_user_id" IS NULL
             AND LOWER("buyer_email") = $3
             AND "status" IN (1, 2, 5)"#,
        [new_user.id.into(), now.into(), email.clone().into()],
    ))
    .await?;
    tx.commit().await?;

    info!("User registered: {}", new_user.username);
    let token = create_jwt(new_user.id, vec![user_role.id], &state.config.jwt)
        .map_err(|_| AppError::auth_failed("Token creation failed"))?;
    Ok(ApiResponse::success(AuthResponse { token }))
}

fn normalize_username(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if !(3..=64).contains(&value.chars().count()) {
        return Err(AppError::validation("用户名长度必须在 3 到 64 个字符之间"));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return Err(AppError::validation(
            "用户名只能包含字母、数字、下划线、短横线和点",
        ));
    }
    Ok(value)
}

fn validate_password(value: &str) -> Result<(), AppError> {
    if !(8..=72).contains(&value.as_bytes().len()) {
        return Err(AppError::validation("密码长度必须在 8 到 72 字节之间"));
    }
    Ok(())
}

#[handler]
pub async fn login(
    payload: JsonBody<AuthPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();

    let user = users::Entity::find()
        .filter(users::Column::Username.eq(&payload.username))
        .one(&state.db)
        .await?;
    let user = user.ok_or_else(|| AppError::not_found("user", None))?;

    let is_valid = verify(&payload.password, &user.password)
        .map_err(|_| AppError::auth_failed("User or password error"))?;
    if !is_valid {
        return Err(AppError::auth_failed("User or password error"));
    }

    let role_ids: Vec<i32> = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|m| m.role_id)
        .collect();

    tracing::info!("User logged in: {}", user.username);
    let token = create_jwt(user.id, role_ids, &state.config.jwt)
        .map_err(|_| AppError::auth_failed("Token creation failed"))?;
    Ok(ApiResponse::success(AuthResponse { token }))
}

#[handler]
pub async fn get_current_user(depot: &mut Depot) -> Result<ApiResponse<users::Model>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let user = users::Entity::find_by_id(claims.user_id)
        .one(&state.db)
        .await?;
    let user = user.ok_or_else(|| AppError::not_found("user", Some(claims.user_id)))?;
    Ok(ApiResponse::success(user))
}

#[handler]
pub async fn get_my_permissions(depot: &mut Depot) -> Result<ApiResponse<Vec<String>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let perms = list_permission_keys_for_roles(&state, &claims.role_ids).await?;
    Ok(ApiResponse::success(perms))
}

#[handler]
pub async fn change_password(
    payload: JsonBody<ChangePasswordPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<bool>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = depot.obtain::<Claims>().unwrap();
    let user = users::Entity::find_by_id(claims.user_id)
        .one(&state.db)
        .await?;
    let user = user.ok_or(AppError::auth_failed("User not found"))?;
    let is_valid = verify(&payload.old_password, &user.password)
        .map_err(|_| AppError::auth_failed("Old password incorrect"))?;
    if !is_valid {
        return Err(AppError::auth_failed("Old password incorrect"));
    }
    let mut active = user.into_active_model();
    active.password = Set(bcrypt::hash(payload.new_password.clone(), 10)?);
    active.updated_at = Set(Utc::now().fixed_offset());
    let _ = active.update(&state.db).await?;
    Ok(ApiResponse::success(true))
}
