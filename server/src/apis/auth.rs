use crate::apis::auth_middleware::Claims;
use crate::core::app::AppState;
use crate::core::constants;
use crate::core::my_error::AppError;
use crate::core::rbac::list_permission_keys_for_roles;
use crate::core::response::ApiResponse;
use crate::utils::jwt::create_jwt;
use bcrypt::verify;
use chrono::Utc;
use data_model::{roles, user_roles, users};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::ToSchema;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Deserialize, ToSchema)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
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
    json: JsonBody<AuthPayload>,
    depot: &mut Depot,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user_exists = users::Entity::find()
        .filter(users::Column::Username.eq(&json.username))
        .one(&state.db)
        .await?;
    if user_exists.is_some() {
        return Err(AppError::user_already_exists());
    }
    let user_role = roles::Entity::find()
        .filter(roles::Column::Name.eq(constants::USER_ROLE))
        .one(&state.db)
        .await?;
    let user_role = user_role.ok_or(AppError::not_found("role", None))?;

    let password = bcrypt::hash(json.password.clone(), 10)?;
    let now = Utc::now().fixed_offset();
    let new_user = users::ActiveModel {
        username: Set(json.username.clone()),
        password: Set(password),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(&state.db)
    .await?;

    user_roles::ActiveModel {
        user_id: Set(new_user.id),
        role_id: Set(user_role.id),
    }
    .insert(&state.db)
    .await?;

    info!("User registered: {}", new_user.username);
    let token = create_jwt(new_user.id, vec![user_role.id], &state.config.jwt)
        .map_err(|_| AppError::auth_failed("Token creation failed"))?;
    Ok(ApiResponse::success(AuthResponse { token }))
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
