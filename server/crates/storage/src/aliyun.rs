use crate::types::{
    StorageAdapter, StorageChannelConfig, StorageError, UploadRequest, UploadResult,
    aliyun_oss_headers, build_aliyun_object_url, canonicalized_aliyun_oss_headers, path_segment,
};
use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header::CONTENT_TYPE;
use sha1::Sha1;
use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct AliyunOssAdapter;

#[async_trait]
impl StorageAdapter for AliyunOssAdapter {
    fn provider(&self) -> &'static str {
        crate::types::PROVIDER_ALIYUN_OSS
    }

    fn label(&self) -> &'static str {
        "Aliyun OSS"
    }

    async fn upload(&self, request: UploadRequest<'_>) -> Result<UploadResult, StorageError> {
        upload_aliyun_oss(request.config, request.object_key, request.body).await
    }
}

async fn upload_aliyun_oss(
    config: &StorageChannelConfig,
    object_key: &str,
    body: &[u8],
) -> Result<UploadResult, StorageError> {
    let encoded_key = object_key
        .split('/')
        .map(path_segment)
        .collect::<Vec<_>>()
        .join("/");
    let url = build_aliyun_object_url(&config.bucket, &config.endpoint, &encoded_key);
    let date = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    let content_type = "application/json";
    let content_md5 = "";
    let oss_headers = aliyun_oss_headers(config);
    let canonicalized_oss_headers = canonicalized_aliyun_oss_headers(&oss_headers);
    let canonicalized_resource = format!("/{}/{}", config.bucket, object_key);
    let string_to_sign = format!(
        "PUT\n{content_md5}\n{content_type}\n{date}\n{canonicalized_oss_headers}{canonicalized_resource}"
    );
    type HmacSha1 = Hmac<Sha1>;
    let mut mac = HmacSha1::new_from_slice(config.access_key_secret.as_bytes())
        .map_err(|error| StorageError::Config(format!("invalid access key secret: {error}")))?;
    mac.update(string_to_sign.as_bytes());
    let signature = base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes());
    let authorization = format!("OSS {}:{}", config.access_key_id, signature);
    let mut request = reqwest::Client::new()
        .put(url)
        .header(CONTENT_TYPE, content_type)
        .header("date", date)
        .header("authorization", authorization);
    for (name, value) in oss_headers {
        request = request.header(name, value);
    }
    let response = request
        .body(body.to_vec())
        .send()
        .await
        .map_err(|error| StorageError::Request(error.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(StorageError::Response(format!(
            "upload failed with status {status}: {text}"
        )));
    }
    let etag = response
        .headers()
        .get("etag")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim_matches('"').to_string())
        .or_else(|| Some(hex::encode(Sha256::digest(body))));
    Ok(UploadResult { etag })
}
