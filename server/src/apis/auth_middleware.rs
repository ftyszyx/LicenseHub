use crate::core::app::*;
use jsonwebtoken::{DecodingKey, Validation, decode};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i32,
    pub role_ids: Vec<i32>,
    pub exp: usize, //过期时间
}

pub fn optional_claims(req: &Request, state: &AppState) -> Result<Option<Claims>, StatusCode> {
    let Some(header) = req.headers().get("Authorization") else {
        return Ok(None);
    };
    let value = header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token = value
        .strip_prefix("Bearer ")
        .filter(|token| !token.trim().is_empty())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt.secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(Some(decoded.claims))
}

#[handler]
pub async fn auth(req: &mut Request, depot: &mut Depot) -> Result<(), StatusCode> {
    let state = depot.obtain::<AppState>().unwrap();
    let claims = optional_claims(req, state)?.ok_or(StatusCode::UNAUTHORIZED)?;
    depot.inject(claims);
    // ctrl.call_next(req, depot, res).await;
    Ok(())
}

#[handler]
pub async fn error_handler(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // 先放行到下游处理
    ctrl.call_next(req, depot, res).await;
    if let Some(code) = res.status_code {
        if code.as_u16() >= 400 {
            tracing::error!("Response status: {}", code);
        }
    }
}
