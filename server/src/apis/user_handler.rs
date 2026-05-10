use crate::apis::auth_middleware::Claims;
use crate::apis::list_api::{ListParamsReq, PagingResponse};
use crate::core::app::*;
use crate::core::constants::{self};
use crate::core::my_error::*;
use crate::core::response::*;
use chrono::Utc;
use data_model::{user_roles, users};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Debug, Validate)]
pub struct UserCreatePayload {
    pub username: String,
    pub password: String,
    pub role_ids: Option<Vec<i32>>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct UserUpdatePayload {
    pub username: Option<String>,
    pub password: Option<String>,
    pub role_ids: Option<Vec<i32>>,
}

#[derive(Serialize )]
pub struct UserWithRoles {
    pub user: users::Model,
    pub role_ids: Vec<i32>,
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchUsersParams {
    #[serde(flatten)]
    pub pagination: ListParamsReq,
    pub username: Option<String>,
    pub id: Option<i32>,
}

// Create User
#[handler]
pub async fn add(
    depot: &mut Depot,
    req: JsonBody<UserCreatePayload>,
) -> Result<ApiResponse<UserWithRoles>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let entity = add_impl(&state, req.into_inner()).await?;
    Ok(ApiResponse::success(entity))
}

pub async fn add_impl(state: &AppState, req: UserCreatePayload) -> Result<UserWithRoles, AppError> {
    let password = bcrypt::hash(req.password, 10)?;
    let now = Utc::now().fixed_offset();
    let active_model = users::ActiveModel {
        username: Set(req.username),
        password: Set(password),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    let entity = active_model.insert(&state.db).await?;

    let mut role_ids = req
        .role_ids
        .unwrap_or_else(|| vec![constants::DEFAULT_ROLE_ID]);
    if role_ids.is_empty() {
        role_ids.push(constants::DEFAULT_ROLE_ID);
    }
    for role_id in &role_ids {
        user_roles::ActiveModel {
            user_id: Set(entity.id),
            role_id: Set(*role_id),
        }
        .insert(&state.db)
        .await?;
    }

    Ok(UserWithRoles {
        user: entity,
        role_ids,
    })
}

// Update User
#[handler]
pub async fn update(
    depot: &mut Depot,
    id: PathParam<i32>,
    req: JsonBody<UserUpdatePayload>,
) -> Result<ApiResponse<UserWithRoles>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user = update_impl(&state, id.into_inner(), req.into_inner()).await?;
    Ok(ApiResponse::success(user))
}

pub async fn update_impl(
    state: &AppState,
    id: i32,
    req: UserUpdatePayload,
) -> Result<UserWithRoles, AppError> {
    let user = users::Entity::find_by_id(id).one(&state.db).await?;
    let user = user.ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;
    let mut user: users::ActiveModel = user.into_active_model();
    if let Some(v) = req.username {
        user.username = Set(v);
    }
    if let Some(password) = req.password {
        let hashed_password = bcrypt::hash(password, 10)?;
        user.password = Set(hashed_password);
    }
    user.updated_at = Set(Utc::now().fixed_offset());
    let user = user.update(&state.db).await?;

    if let Some(role_ids) = req.role_ids {
        user_roles::Entity::delete_many()
            .filter(user_roles::Column::UserId.eq(user.id))
            .exec(&state.db)
            .await?;
        for role_id in &role_ids {
            user_roles::ActiveModel {
                user_id: Set(user.id),
                role_id: Set(*role_id),
            }
            .insert(&state.db)
            .await?;
        }
        Ok(UserWithRoles { user, role_ids })
    } else {
        let role_ids: Vec<i32> = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user.id))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|m| m.role_id)
            .collect();
        Ok(UserWithRoles { user, role_ids })
    }
}

// Delete User
#[handler]
pub async fn delete(depot: &mut Depot, id: PathParam<i32>) -> Result<ApiResponse<()>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let claim = depot.obtain::<Claims>().unwrap();
    let id = id.into_inner();
    //cant delete self
    if id == claim.user_id {
        return Err(AppError::Message("cannot delete self".to_string()));
    }
    delete_impl(&state, id).await?;
    Ok(ApiResponse::success(()))
}

pub async fn delete_impl(state: &AppState, id: i32) -> Result<(), AppError> {
    let user = users::Entity::find_by_id(id).one(&state.db).await?;
    let user = user.ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;
    let _ = user.into_active_model().delete(&state.db).await?;
    Ok(())
}

// Get Users List
#[handler]
pub async fn get_list(
    depot: &mut Depot,
    req: &mut Request,
) -> Result<ApiResponse<PagingResponse<UserWithRoles>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let params = req.parse_queries::<SearchUsersParams>()?;
    let list = get_list_impl(&state, params).await?;
    Ok(ApiResponse::success(list))
}

pub async fn get_list_impl(
    state: &AppState,
    params: SearchUsersParams,
) -> Result<PagingResponse<UserWithRoles>, AppError> {
    let page = params.pagination.page.unwrap_or(1);
    let page_size = params.pagination.page_size.unwrap_or(20);

    let mut query = users::Entity::find().order_by_desc(users::Column::CreatedAt);
    if let Some(v) = params.id {
        query = query.filter(users::Column::Id.eq(v));
    }
    if let Some(v) = params.username {
        query = query.filter(users::Column::Username.like(format!("%{}%", v)));
    }

    let paginator = query.paginate(&state.db, page_size);
    let total = paginator.num_items().await.unwrap_or(0);
    let users_list = paginator.fetch_page(page - 1).await?;
    let mut list = Vec::with_capacity(users_list.len());
    for u in users_list {
        let role_ids: Vec<i32> = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(u.id))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|m| m.role_id)
            .collect();
        list.push(UserWithRoles { user: u, role_ids });
    }
    Ok(PagingResponse { list, total, page })
}

// Get User by ID
#[handler]
pub async fn get_by_id(
    depot: &mut Depot,
    id: PathParam<i32>,
) -> Result<ApiResponse<UserWithRoles>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let user = get_by_id_impl(&state, id.into_inner()).await?;
    Ok(ApiResponse::success(user))
}

pub async fn get_by_id_impl(state: &AppState, id: i32) -> Result<UserWithRoles, AppError> {
    let user = users::Entity::find_by_id(id).one(&state.db).await?;
    let user = user.ok_or_else(|| AppError::not_found("users".to_string(), Some(id)))?;
    let role_ids: Vec<i32> = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(|m| m.role_id)
        .collect();
    Ok(UserWithRoles { user, role_ids })
}
