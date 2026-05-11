use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use data_model::{permissions, role_permissions};
use salvo::{oapi::extract::JsonBody, prelude::*};
use salvo_oapi::extract::PathParam;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct PermissionInfo {
    pub id: i32,
    pub name: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

impl From<permissions::Model> for PermissionInfo {
    fn from(value: permissions::Model) -> Self {
        Self {
            id: value.id,
            name: value.name,
            resource: value.resource,
            action: value.action,
            description: value.description,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct RolePermissionIdsResp {
    pub role_id: i32,
    pub permission_ids: Vec<i32>,
}

#[derive(Deserialize, Debug)]
pub struct SetRolePermissionsReq {
    pub permission_ids: Vec<i32>,
}

#[handler]
pub async fn list_permissions(
    depot: &mut Depot,
) -> Result<ApiResponse<Vec<PermissionInfo>>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let list = permissions::Entity::find().all(&state.db).await?;
    let mut resp: Vec<PermissionInfo> = list.into_iter().map(PermissionInfo::from).collect();
    resp.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(ApiResponse::success(resp))
}

#[handler]
pub async fn get_role_permissions(
    depot: &mut Depot,
    role_id: PathParam<i32>,
) -> Result<ApiResponse<RolePermissionIdsResp>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let role_id = role_id.into_inner();

    let rows = role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.eq(role_id))
        .all(&state.db)
        .await?;

    let permission_ids = rows.into_iter().map(|m| m.permission_id).collect();
    Ok(ApiResponse::success(RolePermissionIdsResp {
        role_id,
        permission_ids,
    }))
}

#[handler]
pub async fn set_role_permissions(
    depot: &mut Depot,
    role_id: PathParam<i32>,
    req: JsonBody<SetRolePermissionsReq>,
) -> Result<ApiResponse<bool>, AppError> {
    let state = depot.obtain::<AppState>().unwrap();
    let role_id = role_id.into_inner();
    let req = req.into_inner();

    // remove existing
    role_permissions::Entity::delete_many()
        .filter(role_permissions::Column::RoleId.eq(role_id))
        .exec(&state.db)
        .await?;

    // insert new
    for pid in req.permission_ids {
        let active = role_permissions::ActiveModel {
            role_id: Set(role_id),
            permission_id: Set(pid),
        };
        let _ = active.insert(&state.db).await?;
    }

    Ok(ApiResponse::success(true))
}
