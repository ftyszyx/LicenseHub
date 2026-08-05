use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use crate::utils::license_signing::{generate_private_key_b64, public_key_b64_from_private_key};
use chrono::Utc;
use data_model::system_settings;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

const STOREFRONT_TITLE_KEY: &str = "storefront_title";
pub const LICENSE_SIGNING_PRIVATE_KEY_KEY: &str = "license_signing_private_key_b64";
const DEFAULT_STOREFRONT_TITLE: &str = "LicenseHub";
pub const DISTRIBUTION_ENABLED_KEY: &str = "distribution_enabled";
pub const DISTRIBUTION_DEFAULT_RATE_BPS_KEY: &str = "distribution_default_rate_bps";
pub const DISTRIBUTION_ATTRIBUTION_DAYS_KEY: &str = "distribution_attribution_days";
pub const DISTRIBUTION_HOLDING_DAYS_KEY: &str = "distribution_holding_days";
pub const DISTRIBUTION_MIN_WITHDRAW_CENTS_KEY: &str = "distribution_min_withdraw_cents";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSettingsInfo {
    pub storefront_title: String,
    pub distribution: DistributionSettingsInfo,
    pub license_signing: LicenseSigningInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSettingsInfo {
    pub enabled: bool,
    pub default_rate_bps: i32,
    pub attribution_days: i32,
    pub holding_days: i32,
    pub min_withdraw_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseSigningInfo {
    pub configured: bool,
    pub key_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key_b64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key_b64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSystemSettingsReq {
    pub storefront_title: String,
    pub distribution_enabled: Option<bool>,
    pub distribution_default_rate_bps: Option<i32>,
    pub distribution_attribution_days: Option<i32>,
    pub distribution_holding_days: Option<i32>,
    pub distribution_min_withdraw_cents: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateLicenseSigningKeyReq {
    pub rotate: bool,
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
    Ok(ApiResponse::success(
        get_site_settings_impl(state, false).await?,
    ))
}

#[handler]
pub async fn get_system_settings(
    depot: &mut Depot,
) -> Result<ApiResponse<SiteSettingsInfo>, AppError> {
    let state = get_state(depot)?;
    Ok(ApiResponse::success(
        get_site_settings_impl(state, true).await?,
    ))
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

#[handler]
pub async fn generate_license_signing_key(
    depot: &mut Depot,
    req: JsonBody<GenerateLicenseSigningKeyReq>,
) -> Result<ApiResponse<SiteSettingsInfo>, AppError> {
    let state = get_state(depot)?;
    Ok(ApiResponse::success(
        generate_license_signing_key_impl(state, req.into_inner()).await?,
    ))
}

pub async fn get_site_settings_impl(
    state: &AppState,
    include_private_key: bool,
) -> Result<SiteSettingsInfo, AppError> {
    Ok(SiteSettingsInfo {
        storefront_title: get_setting_value(state, STOREFRONT_TITLE_KEY)
            .await?
            .unwrap_or_else(|| DEFAULT_STOREFRONT_TITLE.to_string()),
        distribution: get_distribution_settings(state).await?,
        license_signing: get_license_signing_info(state, include_private_key).await?,
    })
}

pub async fn update_system_settings_impl(
    state: &AppState,
    req: UpdateSystemSettingsReq,
) -> Result<SiteSettingsInfo, AppError> {
    let storefront_title = normalize_storefront_title(req.storefront_title)?;
    upsert_setting(state, STOREFRONT_TITLE_KEY, storefront_title).await?;
    if let Some(value) = req.distribution_enabled {
        upsert_setting(state, DISTRIBUTION_ENABLED_KEY, value.to_string()).await?;
    }
    if let Some(value) = req.distribution_default_rate_bps {
        validate_range("distribution_default_rate_bps", value, 0, 10000)?;
        upsert_setting(state, DISTRIBUTION_DEFAULT_RATE_BPS_KEY, value.to_string()).await?;
    }
    if let Some(value) = req.distribution_attribution_days {
        validate_range("distribution_attribution_days", value, 1, 3650)?;
        upsert_setting(state, DISTRIBUTION_ATTRIBUTION_DAYS_KEY, value.to_string()).await?;
    }
    if let Some(value) = req.distribution_holding_days {
        validate_range("distribution_holding_days", value, 0, 3650)?;
        upsert_setting(state, DISTRIBUTION_HOLDING_DAYS_KEY, value.to_string()).await?;
    }
    if let Some(value) = req.distribution_min_withdraw_cents {
        validate_range("distribution_min_withdraw_cents", value, 0, i32::MAX)?;
        upsert_setting(
            state,
            DISTRIBUTION_MIN_WITHDRAW_CENTS_KEY,
            value.to_string(),
        )
        .await?;
    }
    get_site_settings_impl(state, true).await
}

pub async fn get_distribution_settings(
    state: &AppState,
) -> Result<DistributionSettingsInfo, AppError> {
    Ok(DistributionSettingsInfo {
        enabled: setting_bool(state, DISTRIBUTION_ENABLED_KEY, false).await?,
        default_rate_bps: setting_i32(state, DISTRIBUTION_DEFAULT_RATE_BPS_KEY, 2000).await?,
        attribution_days: setting_i32(state, DISTRIBUTION_ATTRIBUTION_DAYS_KEY, 30).await?,
        holding_days: setting_i32(state, DISTRIBUTION_HOLDING_DAYS_KEY, 7).await?,
        min_withdraw_cents: setting_i32(state, DISTRIBUTION_MIN_WITHDRAW_CENTS_KEY, 5000).await?,
    })
}

async fn setting_bool(state: &AppState, key: &str, default: bool) -> Result<bool, AppError> {
    Ok(get_setting_value(state, key)
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or(default))
}

async fn setting_i32(state: &AppState, key: &str, default: i32) -> Result<i32, AppError> {
    Ok(get_setting_value(state, key)
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or(default))
}

pub async fn get_license_signing_private_key_b64(
    state: &AppState,
) -> Result<Option<String>, AppError> {
    get_setting_value(state, LICENSE_SIGNING_PRIVATE_KEY_KEY).await
}

pub async fn get_license_signing_info(
    state: &AppState,
    include_private_key: bool,
) -> Result<LicenseSigningInfo, AppError> {
    let setting = get_setting(state, LICENSE_SIGNING_PRIVATE_KEY_KEY).await?;
    let Some(setting) = setting else {
        return Ok(LicenseSigningInfo {
            configured: false,
            key_id: state.config.license_signing.key_id.clone(),
            public_key_b64: None,
            private_key_b64: None,
            updated_at: None,
        });
    };
    Ok(LicenseSigningInfo {
        configured: true,
        key_id: state.config.license_signing.key_id.clone(),
        public_key_b64: Some(public_key_b64_from_private_key(&setting.value)?),
        private_key_b64: include_private_key.then_some(setting.value),
        updated_at: Some(setting.updated_at.to_rfc3339()),
    })
}

pub async fn generate_license_signing_key_impl(
    state: &AppState,
    req: GenerateLicenseSigningKeyReq,
) -> Result<SiteSettingsInfo, AppError> {
    let existing = get_setting(state, LICENSE_SIGNING_PRIVATE_KEY_KEY).await?;
    if existing.is_some() && !req.rotate {
        return Err(AppError::validation(
            "license signing key already exists; set rotate=true to replace it",
        ));
    }
    let private_key_b64 = generate_private_key_b64()?;
    upsert_setting(state, LICENSE_SIGNING_PRIVATE_KEY_KEY, private_key_b64).await?;
    get_site_settings_impl(state, true).await
}

async fn get_setting(
    state: &AppState,
    key: &str,
) -> Result<Option<system_settings::Model>, AppError> {
    Ok(system_settings::Entity::find_by_id(key.to_string())
        .one(&state.db)
        .await?)
}

async fn get_setting_value(state: &AppState, key: &str) -> Result<Option<String>, AppError> {
    Ok(get_setting(state, key).await?.map(|setting| setting.value))
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

fn validate_range(name: &str, value: i32, min: i32, max: i32) -> Result<(), AppError> {
    if value < min || value > max {
        return Err(AppError::validation(format!(
            "{} must be between {} and {}",
            name, min, max
        )));
    }
    Ok(())
}
