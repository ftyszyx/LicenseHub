use crate::apis::auth_middleware::Claims;
use crate::core::app::AppState;
use crate::core::constants::ADMIN_ROLE_ID;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use data_model::{permissions, role_permissions};
use salvo::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::collections::HashSet;

fn normalize_key(resource: &str, action: &str) -> String {
    format!("{}:{}", resource, action.to_ascii_lowercase())
}

pub async fn list_permission_keys_for_roles(
    state: &AppState,
    role_ids: &[i32],
) -> Result<Vec<String>, AppError> {
    if role_ids.iter().any(|id| *id == ADMIN_ROLE_ID) {
        return Ok(vec![normalize_key("*", "*")]);
    }

    let rows = role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.is_in(role_ids.to_vec()))
        .find_also_related(permissions::Entity)
        .all(&state.db)
        .await?;

    let mut set = HashSet::<String>::new();
    for (_rp, perm) in rows {
        if let Some(p) = perm {
            set.insert(normalize_key(&p.resource, &p.action));
        }
    }

    let mut list: Vec<String> = set.into_iter().collect();
    list.sort();
    Ok(list)
}

fn matches_permission(perm_resource: &str, perm_action: &str, resource: &str, action: &str) -> bool {
    let action = action.to_ascii_uppercase();

    (perm_resource == "*" || perm_resource == resource)
        && (perm_action == "*" || perm_action.eq_ignore_ascii_case(&action))
}

pub async fn ensure_has_permission(
    state: &AppState,
    claims: &Claims,
    resource: &str,
    action: &str,
) -> Result<(), AppError> {
    if claims.role_ids.iter().any(|id| *id == ADMIN_ROLE_ID) {
        return Ok(());
    }

    let rows = role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.is_in(claims.role_ids.clone()))
        .find_also_related(permissions::Entity)
        .all(&state.db)
        .await?;

    for (_rp, perm) in rows {
        if let Some(p) = perm {
            if matches_permission(&p.resource, &p.action, resource, action) {
                return Ok(());
            }
        }
    }

    Err(AppError::Forbidden {
        action: format!("{}:{}", resource, action.to_ascii_lowercase()),
    })
}

#[derive(Clone)]
pub struct RequirePerm {
    resource: &'static str,
    action: &'static str,
}

impl RequirePerm {
    pub fn new(resource: &'static str, action: &'static str) -> Self {
        Self { resource, action }
    }
}

#[salvo::async_trait]
impl Handler for RequirePerm {
    async fn handle(
        &self,
        _req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let state = match depot.obtain::<AppState>() {
            Ok(v) => v,
            Err(_) => {
                res.render(Json(ApiResponse::<String>::error_with_message_and_code(
                    "missing AppState".to_string(),
                    crate::core::constants::APP_INTERNAL_ERROR,
                )));
                ctrl.skip_rest();
                return;
            }
        };

        let claims = match depot.obtain::<Claims>() {
            Ok(v) => v,
            Err(_) => {
                res.render(Json(ApiResponse::<String>::error_with_message_and_code(
                    "unauthorized".to_string(),
                    crate::core::constants::APP_AUTH_FAILED,
                )));
                ctrl.skip_rest();
                return;
            }
        };

        if let Err(e) = ensure_has_permission(state, claims, self.resource, self.action).await {
            res.render(Json(ApiResponse::<String>::error_with_message_and_code(
                e.to_string(),
                e.error_code(),
            )));
            ctrl.skip_rest();
            return;
        }

        ctrl.call_next(_req, depot, res).await;
    }
}
