use crate::core::app::AppState;
use crate::core::my_error::AppError;
use crate::core::response::ApiResponse;
use crate::mailer::{EmailServiceConfig, send_email_code};
use crate::utils::license_signing::{generate_private_key_b64, public_key_b64_from_private_key};
use chrono::Utc;
use data_model::system_settings;
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

const STOREFRONT_TITLE_KEY: &str = "storefront_title";
pub const LICENSE_SIGNING_PRIVATE_KEY_KEY: &str = "license_signing_private_key_b64";
const DEFAULT_STOREFRONT_TITLE: &str = "LicenseHub";
pub const DISTRIBUTION_ENABLED_KEY: &str = "distribution_enabled";
pub const DISTRIBUTION_REFERRER_BINDING_ENABLED_KEY: &str = "distribution_referrer_binding_enabled";
pub const DISTRIBUTION_DEFAULT_RATE_BPS_KEY: &str = "distribution_default_rate_bps";
pub const DISTRIBUTION_ATTRIBUTION_DAYS_KEY: &str = "distribution_attribution_days";
pub const DISTRIBUTION_HOLDING_DAYS_KEY: &str = "distribution_holding_days";
pub const DISTRIBUTION_MIN_WITHDRAW_CENTS_KEY: &str = "distribution_min_withdraw_cents";
pub const REGISTRATION_ENABLED_KEY: &str = "registration_enabled";
const EMAIL_SERVICE_MODE_KEY: &str = "email_service_mode";
const EMAIL_FROM_KEY: &str = "email_from";
const EMAIL_SMTP_HOST_KEY: &str = "email_smtp_host";
const EMAIL_SMTP_PORT_KEY: &str = "email_smtp_port";
const EMAIL_SMTP_USERNAME_KEY: &str = "email_smtp_username";
const EMAIL_SMTP_PASSWORD_KEY: &str = "email_smtp_password";
const EMAIL_SMTP_TLS_MODE_KEY: &str = "email_smtp_tls_mode";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSettingsInfo {
    pub storefront_title: String,
    pub registration_enabled: bool,
    pub distribution: DistributionSettingsInfo,
    pub license_signing: LicenseSigningInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailSettingsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSettingsInfo {
    pub mode: String,
    pub from: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password_set: bool,
    pub smtp_tls_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionSettingsInfo {
    pub enabled: bool,
    pub referrer_binding_enabled: bool,
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
    pub registration_enabled: Option<bool>,
    pub distribution_enabled: Option<bool>,
    pub distribution_referrer_binding_enabled: Option<bool>,
    pub distribution_default_rate_bps: Option<i32>,
    pub distribution_attribution_days: Option<i32>,
    pub distribution_holding_days: Option<i32>,
    pub distribution_min_withdraw_cents: Option<i32>,
    pub email_service_mode: Option<String>,
    pub email_from: Option<String>,
    pub email_smtp_host: Option<String>,
    pub email_smtp_port: Option<u16>,
    pub email_smtp_username: Option<String>,
    pub email_smtp_password: Option<String>,
    pub email_smtp_tls_mode: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestEmailReq {
    pub email: String,
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

#[handler]
pub async fn send_test_email(
    depot: &mut Depot,
    req: JsonBody<TestEmailReq>,
) -> Result<ApiResponse<bool>, AppError> {
    let state = get_state(depot)?;
    let email = normalize_email(&req.email)?;
    let config = get_email_service_config(state).await?;
    validate_email_service_config(&config)?;
    send_email_code(&config, &email, "123456", 10).await?;
    Ok(ApiResponse::success(true))
}

pub async fn get_site_settings_impl(
    state: &AppState,
    include_private_key: bool,
) -> Result<SiteSettingsInfo, AppError> {
    Ok(SiteSettingsInfo {
        storefront_title: get_setting_value(state, STOREFRONT_TITLE_KEY)
            .await?
            .unwrap_or_else(|| DEFAULT_STOREFRONT_TITLE.to_string()),
        registration_enabled: get_registration_enabled(state).await?,
        distribution: get_distribution_settings(state).await?,
        license_signing: get_license_signing_info(state, include_private_key).await?,
        email: if include_private_key {
            Some(get_email_settings_info(state).await?)
        } else {
            None
        },
    })
}

pub async fn update_system_settings_impl(
    state: &AppState,
    req: UpdateSystemSettingsReq,
) -> Result<SiteSettingsInfo, AppError> {
    let storefront_title = normalize_storefront_title(req.storefront_title)?;
    let mut updates = vec![(STOREFRONT_TITLE_KEY, storefront_title)];
    if let Some(value) = req.registration_enabled {
        if value && state.config.email_code_secret.is_none() {
            return Err(AppError::validation(
                "EMAIL_CODE_SECRET must be configured before registration can be enabled",
            ));
        }
        updates.push((REGISTRATION_ENABLED_KEY, value.to_string()));
    }
    if let Some(value) = req.distribution_enabled {
        updates.push((DISTRIBUTION_ENABLED_KEY, value.to_string()));
    }
    if let Some(value) = req.distribution_referrer_binding_enabled {
        updates.push((DISTRIBUTION_REFERRER_BINDING_ENABLED_KEY, value.to_string()));
    }
    if let Some(value) = req.distribution_default_rate_bps {
        validate_range("distribution_default_rate_bps", value, 0, 10000)?;
        updates.push((DISTRIBUTION_DEFAULT_RATE_BPS_KEY, value.to_string()));
    }
    if let Some(value) = req.distribution_attribution_days {
        validate_range("distribution_attribution_days", value, 1, 3650)?;
        updates.push((DISTRIBUTION_ATTRIBUTION_DAYS_KEY, value.to_string()));
    }
    if let Some(value) = req.distribution_holding_days {
        validate_range("distribution_holding_days", value, 0, 3650)?;
        updates.push((DISTRIBUTION_HOLDING_DAYS_KEY, value.to_string()));
    }
    if let Some(value) = req.distribution_min_withdraw_cents {
        validate_range("distribution_min_withdraw_cents", value, 0, i32::MAX)?;
        updates.push((DISTRIBUTION_MIN_WITHDRAW_CENTS_KEY, value.to_string()));
    }

    let mut email = get_email_service_config(state).await?;
    if let Some(value) = req.email_service_mode {
        email.mode = normalize_enum("email_service_mode", value, &["log", "smtp"])?;
        updates.push((EMAIL_SERVICE_MODE_KEY, email.mode.clone()));
    }
    if let Some(value) = req.email_from {
        email.from = normalize_required("email_from", value, 320)?;
        updates.push((EMAIL_FROM_KEY, email.from.clone()));
    }
    if let Some(value) = req.email_smtp_host {
        email.smtp_host = normalize_optional(value, 255)?;
        updates.push((EMAIL_SMTP_HOST_KEY, email.smtp_host.clone()));
    }
    if let Some(value) = req.email_smtp_port {
        if value == 0 {
            return Err(AppError::validation(
                "email_smtp_port must be greater than 0",
            ));
        }
        email.smtp_port = value;
        updates.push((EMAIL_SMTP_PORT_KEY, value.to_string()));
    }
    if let Some(value) = req.email_smtp_username {
        email.smtp_username = normalize_optional(value, 320)?;
        updates.push((EMAIL_SMTP_USERNAME_KEY, email.smtp_username.clone()));
    }
    if let Some(value) = req.email_smtp_password.filter(|value| !value.is_empty()) {
        if value.len() > 2048 {
            return Err(AppError::validation("email_smtp_password is too long"));
        }
        email.smtp_password = value.clone();
        updates.push((EMAIL_SMTP_PASSWORD_KEY, value));
    }
    if let Some(value) = req.email_smtp_tls_mode {
        email.smtp_tls_mode =
            normalize_enum("email_smtp_tls_mode", value, &["starttls", "tls", "none"])?;
        updates.push((EMAIL_SMTP_TLS_MODE_KEY, email.smtp_tls_mode.clone()));
    }
    validate_email_service_config(&email)?;

    let tx = state.db.begin().await?;
    for (key, value) in updates {
        upsert_setting(&tx, key, value).await?;
    }
    tx.commit().await?;

    get_site_settings_impl(state, true).await
}

pub async fn get_distribution_settings(
    state: &AppState,
) -> Result<DistributionSettingsInfo, AppError> {
    Ok(DistributionSettingsInfo {
        enabled: setting_bool(state, DISTRIBUTION_ENABLED_KEY, false).await?,
        referrer_binding_enabled: setting_bool(
            state,
            DISTRIBUTION_REFERRER_BINDING_ENABLED_KEY,
            false,
        )
        .await?,
        default_rate_bps: setting_i32(state, DISTRIBUTION_DEFAULT_RATE_BPS_KEY, 2000).await?,
        attribution_days: setting_i32(state, DISTRIBUTION_ATTRIBUTION_DAYS_KEY, 30).await?,
        holding_days: setting_i32(state, DISTRIBUTION_HOLDING_DAYS_KEY, 7).await?,
        min_withdraw_cents: setting_i32(state, DISTRIBUTION_MIN_WITHDRAW_CENTS_KEY, 5000).await?,
    })
}

pub async fn get_registration_enabled(state: &AppState) -> Result<bool, AppError> {
    setting_bool(state, REGISTRATION_ENABLED_KEY, state.config.register_open).await
}

pub async fn get_email_service_config(state: &AppState) -> Result<EmailServiceConfig, AppError> {
    Ok(EmailServiceConfig {
        mode: get_setting_value(state, EMAIL_SERVICE_MODE_KEY)
            .await?
            .unwrap_or_else(|| "log".to_string()),
        from: get_setting_value(state, EMAIL_FROM_KEY)
            .await?
            .unwrap_or_else(|| "LicenseHub <no-reply@example.com>".to_string()),
        smtp_host: get_setting_value(state, EMAIL_SMTP_HOST_KEY)
            .await?
            .unwrap_or_default(),
        smtp_port: get_setting_value(state, EMAIL_SMTP_PORT_KEY)
            .await?
            .and_then(|value| value.parse().ok())
            .unwrap_or(587),
        smtp_username: get_setting_value(state, EMAIL_SMTP_USERNAME_KEY)
            .await?
            .unwrap_or_default(),
        smtp_password: get_setting_value(state, EMAIL_SMTP_PASSWORD_KEY)
            .await?
            .unwrap_or_default(),
        smtp_tls_mode: get_setting_value(state, EMAIL_SMTP_TLS_MODE_KEY)
            .await?
            .unwrap_or_else(|| "starttls".to_string()),
    })
}

async fn get_email_settings_info(state: &AppState) -> Result<EmailSettingsInfo, AppError> {
    let config = get_email_service_config(state).await?;
    Ok(EmailSettingsInfo {
        mode: config.mode,
        from: config.from,
        smtp_host: config.smtp_host,
        smtp_port: config.smtp_port,
        smtp_username: config.smtp_username,
        smtp_password_set: !config.smtp_password.is_empty(),
        smtp_tls_mode: config.smtp_tls_mode,
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
    upsert_setting(&state.db, LICENSE_SIGNING_PRIVATE_KEY_KEY, private_key_b64).await?;
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

async fn upsert_setting<C>(db: &C, key: &str, value: String) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let now = Utc::now().fixed_offset();
    let existing = system_settings::Entity::find()
        .filter(system_settings::Column::Key.eq(key))
        .one(db)
        .await?;

    match existing {
        Some(setting) => {
            let mut active: system_settings::ActiveModel = setting.into();
            active.value = Set(value);
            active.updated_at = Set(now);
            active.update(db).await?;
        }
        None => {
            system_settings::ActiveModel {
                key: Set(key.to_string()),
                value: Set(value),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(db)
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

fn normalize_email(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_ascii_lowercase();
    if value.len() > 320 || value.parse::<email_address::EmailAddress>().is_err() {
        return Err(AppError::business_logic("EMAIL_INVALID", "邮箱格式不正确"));
    }
    Ok(value)
}

fn normalize_required(name: &str, value: String, max: usize) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(AppError::validation(format!("{name} is required")));
    }
    if value.len() > max {
        return Err(AppError::validation(format!("{name} is too long")));
    }
    Ok(value)
}

fn normalize_optional(value: String, max: usize) -> Result<String, AppError> {
    let value = value.trim().to_string();
    if value.len() > max {
        return Err(AppError::validation("setting value is too long"));
    }
    Ok(value)
}

fn normalize_enum(name: &str, value: String, allowed: &[&str]) -> Result<String, AppError> {
    let value = value.trim().to_ascii_lowercase();
    if !allowed.contains(&value.as_str()) {
        return Err(AppError::validation(format!("invalid {name}")));
    }
    Ok(value)
}

fn validate_email_service_config(config: &EmailServiceConfig) -> Result<(), AppError> {
    normalize_enum("email_service_mode", config.mode.clone(), &["log", "smtp"])?;
    config
        .from
        .parse::<lettre::message::Mailbox>()
        .map_err(|_| AppError::validation("email_from must be a valid mailbox"))?;
    normalize_enum(
        "email_smtp_tls_mode",
        config.smtp_tls_mode.clone(),
        &["starttls", "tls", "none"],
    )?;
    if config.mode == "smtp"
        && (config.smtp_host.trim().is_empty()
            || config.smtp_username.trim().is_empty()
            || config.smtp_password.is_empty())
    {
        return Err(AppError::validation(
            "SMTP host, username and password are required in smtp mode",
        ));
    }
    Ok(())
}
