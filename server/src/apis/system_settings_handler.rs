use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use chrono::Utc;
use data_model::system_settings;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

const STOREFRONT_TITLE_KEY: &str = "storefront_title";
const DEFAULT_STOREFRONT_TITLE: &str = "LicenseHub";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSettingsInfo {
    pub storefront_title: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSystemSettingsReq {
    pub storefront_title: String,
}

fn get_state(depot: &mut Depot) -> Result<&AppState, AppError> {
    depot
        .obtain::<AppState>()
        .map_err(|_| AppError::InternalError {
            message: "missing AppState in depot".to_string(),
        })
}

#[handler]
pub async fn get_public_site_settings(
    depot: &mut Depot,
) -> Result<ApiResponse<SiteSettingsInfo>, AppError> {
    let state = get_state(depot)?;
    Ok(ApiResponse::success(get_site_settings_impl(state).await?))
}

#[handler]
pub async fn get_system_settings(
    depot: &mut Depot,
) -> Result<ApiResponse<SiteSettingsInfo>, AppError> {
    let state = get_state(depot)?;
    Ok(ApiResponse::success(get_site_settings_impl(state).await?))
}

#[handler]
pub async fn update_system_settings(
    depot: &mut Depot,
    req: JsonBody<UpdateSystemSettingsReq>,
) -> Result<ApiResponse<SiteSettingsInfo>, AppError> {
    let state = get_state(depot)?;
    let req = req.into_inner();
    Ok(ApiResponse::success(
        update_system_settings_impl(state, req).await?,
    ))
}

pub async fn get_site_settings_impl(state: &AppState) -> Result<SiteSettingsInfo, AppError> {
    Ok(SiteSettingsInfo {
        storefront_title: get_setting_value(state, STOREFRONT_TITLE_KEY)
            .await?
            .unwrap_or_else(|| DEFAULT_STOREFRONT_TITLE.to_string()),
    })
}

pub async fn update_system_settings_impl(
    state: &AppState,
    req: UpdateSystemSettingsReq,
) -> Result<SiteSettingsInfo, AppError> {
    let storefront_title = normalize_storefront_title(req.storefront_title)?;
    upsert_setting(state, STOREFRONT_TITLE_KEY, storefront_title).await?;
    get_site_settings_impl(state).await
}

async fn get_setting_value(state: &AppState, key: &str) -> Result<Option<String>, AppError> {
    Ok(system_settings::Entity::find_by_id(key.to_string())
        .one(&state.db)
        .await?
        .map(|setting| setting.value))
}

async fn upsert_setting(state: &AppState, key: &str, value: String) -> Result<(), AppError> {
    let now = Utc::now().fixed_offset();
    let existing = system_settings::Entity::find()
        .filter(system_settings::Column::Key.eq(key))
        .one(&state.db)
        .await?;

    match existing {
        Some(setting) => {
            let mut active: system_settings::ActiveModel = setting.into();
            active.value = Set(value);
            active.updated_at = Set(now);
            active.update(&state.db).await?;
        }
        None => {
            system_settings::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(&state.db)
            .await?;
        }
    }

    Ok(())
}

fn normalize_storefront_title(value: String) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(AppError::validation("storefront_title is required"));
    }
    if value.chars().count() > 80 {
        return Err(AppError::validation(
            "storefront_title must be at most 80 characters",
        ));
    }
    Ok(value)
}
