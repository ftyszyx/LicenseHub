use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const PROVIDER_ALIYUN_OSS: &str = "aliyun_oss";
pub const PROVIDER_CLOUDFLARE_R2: &str = "cloudflare_r2";
pub const PROVIDER_MOCK: &str = "mock";
pub const DEFAULT_PREFIX: &str = "apps";

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage config error: {0}")]
    Config(String),
    #[error("storage request error: {0}")]
    Request(String),
    #[error("storage response error: {0}")]
    Response(String),
    #[error("storage signature error: {0}")]
    Signature(String),
    #[error("unsupported storage provider: {0}")]
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChannelConfig {
    pub bucket: String,
    pub region: Option<String>,
    pub endpoint: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub public_base_url: String,
    pub prefix: String,
    pub storage_class: Option<String>,
    pub object_acl: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UploadRequest<'a> {
    pub config: &'a StorageChannelConfig,
    pub object_key: &'a str,
    pub body: &'a [u8],
    pub content_type: &'a str,
    pub private: bool,
}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DownloadRequest<'a> {
    pub config: &'a StorageChannelConfig,
    pub object_key: &'a str,
}

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub body: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteRequest<'a> {
    pub config: &'a StorageChannelConfig,
    pub object_key: &'a str,
}

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    fn provider(&self) -> &'static str;
    fn label(&self) -> &'static str;

    async fn upload(&self, request: UploadRequest<'_>) -> Result<UploadResult, StorageError>;

    async fn download(&self, request: DownloadRequest<'_>) -> Result<DownloadResult, StorageError>;

    async fn delete(&self, request: DeleteRequest<'_>) -> Result<(), StorageError>;
}

pub fn normalize_storage_provider(provider: &str) -> Result<String, StorageError> {
    let provider = provider.trim().to_ascii_lowercase();
    match provider.as_str() {
        PROVIDER_ALIYUN_OSS | PROVIDER_CLOUDFLARE_R2 => Ok(provider),
        _ => Err(StorageError::Unsupported(provider)),
    }
}

pub fn normalize_storage_config(
    provider: &str,
    config: StorageChannelConfig,
) -> Result<StorageChannelConfig, StorageError> {
    let provider = normalize_storage_provider(provider)?;
    let bucket = normalize_required_text(config.bucket, "bucket")?;
    let endpoint = normalize_storage_endpoint(&provider, config.endpoint)?;
    let prefix = normalize_prefix(config.prefix);
    let public_base_url =
        normalize_public_base_url(&provider, &bucket, &endpoint, config.public_base_url)?;
    let storage_class = normalize_storage_class(&provider, config.storage_class)?;
    let object_acl = normalize_object_acl(&provider, config.object_acl)?;

    Ok(StorageChannelConfig {
        bucket,
        region: config
            .region
            .and_then(|value| non_empty_string(value.trim().to_string())),
        endpoint,
        access_key_id: normalize_required_text(config.access_key_id, "access_key_id")?,
        access_key_secret: normalize_required_text(config.access_key_secret, "access_key_secret")?,
        public_base_url,
        prefix,
        storage_class,
        object_acl,
    })
}

pub fn parse_storage_config(
    provider: &str,
    config: &Value,
) -> Result<StorageChannelConfig, StorageError> {
    ensure_config_object(config)?;
    normalize_storage_config(
        provider,
        StorageChannelConfig {
            bucket: config_text(config, "bucket"),
            region: non_empty_string(config_text(config, "region")),
            endpoint: config_text(config, "endpoint"),
            access_key_id: config_text(config, "access_key_id"),
            access_key_secret: config_text(config, "access_key_secret"),
            public_base_url: config_text(config, "public_base_url"),
            prefix: config_text(config, "prefix"),
            storage_class: non_empty_string(config_text(config, "storage_class")),
            object_acl: non_empty_string(config_text(config, "object_acl")),
        },
    )
}

fn ensure_config_object(config: &Value) -> Result<(), StorageError> {
    if config.is_object() {
        Ok(())
    } else {
        Err(StorageError::Config(
            "storage channel config must be an object".to_string(),
        ))
    }
}

fn config_text(config: &Value, key: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn normalize_required_text(value: String, field: &str) -> Result<String, StorageError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(StorageError::Config(format!(
            "storage channel config field '{field}' is required"
        )))
    } else {
        Ok(value)
    }
}

pub fn normalize_prefix(prefix: String) -> String {
    let prefix = prefix.trim().trim_matches('/').to_string();
    if prefix.is_empty() {
        DEFAULT_PREFIX.to_string()
    } else {
        prefix
    }
}

pub fn build_object_key(prefix: &str, app_id: &str) -> String {
    format!(
        "{}/{}/latest.json",
        normalize_prefix(prefix.to_string()),
        app_id.trim_matches('/')
    )
}

pub fn build_public_url(public_base_url: &str, object_key: &str) -> String {
    format!(
        "{}/{}",
        public_base_url.trim_end_matches('/'),
        object_key
            .split('/')
            .map(path_segment)
            .collect::<Vec<_>>()
            .join("/")
    )
}

pub fn path_segment(segment: &str) -> String {
    urlencoding::encode(segment).into_owned()
}

fn normalize_storage_endpoint(provider: &str, endpoint: String) -> Result<String, StorageError> {
    let endpoint = endpoint.trim().trim_end_matches('/').to_string();
    if endpoint.starts_with("mock://") {
        return Ok(endpoint);
    }
    let endpoint = endpoint
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string();
    if !endpoint.is_empty() {
        return Ok(if provider == PROVIDER_ALIYUN_OSS {
            format!("https://{}", endpoint)
        } else {
            format!("https://{}", endpoint)
        });
    }
    Err(StorageError::Config(
        "storage channel config field 'endpoint' is required".to_string(),
    ))
}

fn normalize_public_base_url(
    provider: &str,
    bucket: &str,
    endpoint: &str,
    public_base_url: String,
) -> Result<String, StorageError> {
    let public_base_url = public_base_url.trim_end_matches('/').to_string();
    if !public_base_url.is_empty() {
        return Ok(normalize_public_url_scheme(public_base_url));
    }
    if provider == PROVIDER_ALIYUN_OSS {
        return Ok(build_aliyun_public_base_url(bucket, endpoint));
    }
    Err(StorageError::Config(
        "storage channel config field 'public_base_url' is required".to_string(),
    ))
}

pub fn build_aliyun_public_base_url(bucket: &str, endpoint: &str) -> String {
    format!("https://{}", aliyun_bucket_host(bucket, endpoint))
}

pub fn normalize_public_url_scheme(public_base_url: String) -> String {
    if public_base_url.starts_with("https://") || public_base_url.starts_with("http://") {
        public_base_url
    } else {
        format!("https://{}", public_base_url)
    }
}

fn normalize_storage_class(
    provider: &str,
    storage_class: Option<String>,
) -> Result<Option<String>, StorageError> {
    if provider != PROVIDER_ALIYUN_OSS {
        return Ok(None);
    }
    let storage_class = storage_class
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    let storage_class = if storage_class.is_empty() {
        "standard".to_string()
    } else {
        storage_class
    };
    match storage_class.as_str() {
        "standard" | "ia" | "archive" | "cold_archive" | "deep_cold_archive" => {
            Ok(Some(storage_class))
        }
        _ => Err(StorageError::Config(
            "storage_class is not supported".to_string(),
        )),
    }
}

fn normalize_object_acl(
    provider: &str,
    object_acl: Option<String>,
) -> Result<Option<String>, StorageError> {
    if provider != PROVIDER_ALIYUN_OSS {
        return Ok(None);
    }
    let object_acl = object_acl.unwrap_or_default().trim().to_ascii_lowercase();
    let object_acl = if object_acl.is_empty() {
        "public-read".to_string()
    } else {
        object_acl
    };
    match object_acl.as_str() {
        "default" | "private" | "public-read" | "public-read-write" => Ok(Some(object_acl)),
        _ => Err(StorageError::Config(
            "object_acl is not supported".to_string(),
        )),
    }
}

pub fn aliyun_storage_class_header(storage_class: Option<&str>) -> Option<&'static str> {
    match storage_class {
        Some("standard") | None => Some("Standard"),
        Some("ia") => Some("IA"),
        Some("archive") => Some("Archive"),
        Some("cold_archive") => Some("ColdArchive"),
        Some("deep_cold_archive") => Some("DeepColdArchive"),
        _ => None,
    }
}

pub fn aliyun_object_acl_header(object_acl: Option<&str>) -> Option<&'static str> {
    match object_acl {
        Some("default") | None => None,
        Some("private") => Some("private"),
        Some("public-read") => Some("public-read"),
        Some("public-read-write") => Some("public-read-write"),
        _ => None,
    }
}

pub fn aliyun_oss_headers(config: &StorageChannelConfig) -> Vec<(&'static str, &'static str)> {
    let mut headers = Vec::new();
    if let Some(object_acl) = aliyun_object_acl_header(config.object_acl.as_deref()) {
        headers.push(("x-oss-object-acl", object_acl));
    }
    if let Some(storage_class) = aliyun_storage_class_header(config.storage_class.as_deref()) {
        headers.push(("x-oss-storage-class", storage_class));
    }
    headers.sort_by_key(|(name, _)| *name);
    headers
}

pub fn canonicalized_aliyun_oss_headers(headers: &[(&str, &str)]) -> String {
    headers
        .iter()
        .map(|(name, value)| format!("{}:{}\n", name.to_ascii_lowercase(), value.trim()))
        .collect()
}

pub fn build_aliyun_object_url(bucket: &str, endpoint: &str, encoded_key: &str) -> String {
    format!(
        "https://{}/{}",
        aliyun_bucket_host(bucket, endpoint),
        encoded_key.trim_start_matches('/')
    )
}

pub fn aliyun_bucket_host(bucket: &str, endpoint: &str) -> String {
    let bucket = bucket.trim();
    let endpoint = endpoint
        .trim()
        .trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    if endpoint.starts_with(&format!("{}.", bucket)) {
        endpoint.to_string()
    } else {
        format!("{}.{}", bucket, endpoint)
    }
}

fn non_empty_string(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliyun_object_url_uses_bucket_third_level_domain() {
        assert_eq!(
            build_aliyun_object_url(
                "bytefuse",
                "https://oss-cn-guangzhou.aliyuncs.com",
                "apps/app/latest.json"
            ),
            "https://bytefuse.oss-cn-guangzhou.aliyuncs.com/apps/app/latest.json"
        );
    }

    #[test]
    fn aliyun_public_base_url_does_not_duplicate_bucket_domain() {
        assert_eq!(
            build_aliyun_public_base_url(
                "bytefuse",
                "https://bytefuse.oss-cn-guangzhou.aliyuncs.com"
            ),
            "https://bytefuse.oss-cn-guangzhou.aliyuncs.com"
        );
    }

    #[test]
    fn aliyun_object_acl_defaults_to_public_read() {
        assert_eq!(
            normalize_object_acl(PROVIDER_ALIYUN_OSS, Some("".to_string())).unwrap(),
            Some("public-read".to_string())
        );
        assert_eq!(
            normalize_object_acl(PROVIDER_ALIYUN_OSS, Some("default".to_string())).unwrap(),
            Some("default".to_string())
        );
        assert!(aliyun_object_acl_header(Some("default")).is_none());
    }

    #[test]
    fn aliyun_canonicalized_headers_include_acl_in_order() {
        let config = StorageChannelConfig {
            bucket: "bytefuse".to_string(),
            region: None,
            endpoint: "https://oss-cn-guangzhou.aliyuncs.com".to_string(),
            access_key_id: "ak".to_string(),
            access_key_secret: "sk".to_string(),
            public_base_url: "https://bytefuse.oss-cn-guangzhou.aliyuncs.com".to_string(),
            prefix: "apps".to_string(),
            storage_class: Some("ia".to_string()),
            object_acl: Some("public-read".to_string()),
        };

        assert_eq!(
            canonicalized_aliyun_oss_headers(&aliyun_oss_headers(&config)),
            "x-oss-object-acl:public-read\nx-oss-storage-class:IA\n"
        );
    }

    #[test]
    fn public_base_url_adds_https_when_scheme_is_missing() {
        assert_eq!(
            normalize_public_base_url(
                PROVIDER_CLOUDFLARE_R2,
                "apphub",
                "https://account.r2.cloudflarestorage.com",
                "apphub.1postpro.com/".to_string(),
            )
            .unwrap(),
            "https://apphub.1postpro.com"
        );
        assert_eq!(
            normalize_public_base_url(
                PROVIDER_CLOUDFLARE_R2,
                "apphub",
                "https://account.r2.cloudflarestorage.com",
                "apphub.1postpro.com/apphub/".to_string(),
            )
            .unwrap(),
            "https://apphub.1postpro.com/apphub"
        );
    }

    #[test]
    fn public_url_keeps_custom_prefix_path() {
        assert_eq!(
            build_public_url(
                "https://apphub.1postpro.com/apphub",
                "apps/NAE2W1U7444J/latest.json"
            ),
            "https://apphub.1postpro.com/apphub/apps/NAE2W1U7444J/latest.json"
        );
    }
}
