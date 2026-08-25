use crate::types::{
    DeleteRequest, DownloadRequest, DownloadResult, StorageAdapter, StorageChannelConfig,
    StorageError, UploadRequest, UploadResult, path_segment,
};
use async_trait::async_trait;
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{SignableBody, SignableRequest, SigningSettings, sign};
use aws_sigv4::sign::v4;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use sha2::{Digest, Sha256};
use std::time::SystemTime;

#[derive(Default)]
pub struct CloudflareR2Adapter;

#[async_trait]
impl StorageAdapter for CloudflareR2Adapter {
    fn provider(&self) -> &'static str {
        crate::types::PROVIDER_CLOUDFLARE_R2
    }

    fn label(&self) -> &'static str {
        "Cloudflare R2"
    }

    async fn upload(&self, request: UploadRequest<'_>) -> Result<UploadResult, StorageError> {
        upload_s3_compatible(
            request.config,
            request.object_key,
            request.body,
            request.content_type,
        )
        .await
    }

    async fn download(&self, request: DownloadRequest<'_>) -> Result<DownloadResult, StorageError> {
        download_s3_compatible(request.config, request.object_key).await
    }

    async fn delete(&self, request: DeleteRequest<'_>) -> Result<(), StorageError> {
        delete_s3_compatible(request.config, request.object_key).await
    }
}

async fn upload_s3_compatible(
    config: &StorageChannelConfig,
    object_key: &str,
    body: &[u8],
    content_type: &str,
) -> Result<UploadResult, StorageError> {
    let region = config.region.as_deref().unwrap_or("auto");
    let payload_sha256 = hex::encode(Sha256::digest(body));
    let url = build_s3_compatible_object_url(&config.endpoint, &config.bucket, object_key);
    let credentials = Credentials::new(
        config.access_key_id.clone(),
        config.access_key_secret.clone(),
        None,
        None,
        "licensehub",
    );
    let identity = credentials.into();
    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name("s3")
        .time(SystemTime::now())
        .settings(SigningSettings::default())
        .build()
        .map_err(|error| {
            StorageError::Signature(format!("failed to build signing params: {error}"))
        })?
        .into();
    let signable = SignableRequest::new(
        "PUT",
        &url,
        [
            ("content-type", content_type),
            ("x-amz-content-sha256", payload_sha256.as_str()),
        ]
        .into_iter(),
        SignableBody::Bytes(body),
    )
    .map_err(|error| {
        StorageError::Signature(format!("failed to build signable request: {error}"))
    })?;
    let (instructions, _signature) = sign(signable, &params)
        .map_err(|error| {
            StorageError::Signature(format!("failed to sign storage request: {error}"))
        })?
        .into_parts();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(content_type).map_err(|error| {
            StorageError::Signature(format!("invalid content type header value: {error}"))
        })?,
    );
    headers.insert(
        HeaderName::from_static("x-amz-content-sha256"),
        HeaderValue::from_str(&payload_sha256).map_err(|error| {
            StorageError::Signature(format!("invalid payload hash header value: {error}"))
        })?,
    );
    for (name, value) in instructions.headers() {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                StorageError::Signature(format!("invalid signed header name: {error}"))
            })?,
            HeaderValue::from_str(value).map_err(|error| {
                StorageError::Signature(format!("invalid signed header value: {error}"))
            })?,
        );
    }
    let response = reqwest::Client::new()
        .put(url)
        .headers(headers)
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
        .map(|value| value.trim_matches('"').to_string());
    Ok(UploadResult { etag })
}

async fn download_s3_compatible(
    config: &StorageChannelConfig,
    object_key: &str,
) -> Result<DownloadResult, StorageError> {
    let region = config.region.as_deref().unwrap_or("auto");
    let payload_sha256 = hex::encode(Sha256::digest([]));
    let url = build_s3_compatible_object_url(&config.endpoint, &config.bucket, object_key);
    let credentials = Credentials::new(
        config.access_key_id.clone(),
        config.access_key_secret.clone(),
        None,
        None,
        "licensehub",
    );
    let identity = credentials.into();
    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name("s3")
        .time(SystemTime::now())
        .settings(SigningSettings::default())
        .build()
        .map_err(|error| {
            StorageError::Signature(format!("failed to build signing params: {error}"))
        })?
        .into();
    let signable = SignableRequest::new(
        "GET",
        &url,
        [("x-amz-content-sha256", payload_sha256.as_str())].into_iter(),
        SignableBody::Bytes(&[]),
    )
    .map_err(|error| {
        StorageError::Signature(format!("failed to build signable request: {error}"))
    })?;
    let (instructions, _signature) = sign(signable, &params)
        .map_err(|error| {
            StorageError::Signature(format!("failed to sign storage request: {error}"))
        })?
        .into_parts();
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-amz-content-sha256"),
        HeaderValue::from_str(&payload_sha256).map_err(|error| {
            StorageError::Signature(format!("invalid payload hash header value: {error}"))
        })?,
    );
    for (name, value) in instructions.headers() {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                StorageError::Signature(format!("invalid signed header name: {error}"))
            })?,
            HeaderValue::from_str(value).map_err(|error| {
                StorageError::Signature(format!("invalid signed header value: {error}"))
            })?,
        );
    }
    let response = reqwest::Client::new()
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|error| StorageError::Request(error.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(StorageError::Response(format!(
            "download failed with status {status}: {text}"
        )));
    }
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = response
        .bytes()
        .await
        .map_err(|error| {
            StorageError::Response(format!("failed to read downloaded object: {error}"))
        })?
        .to_vec();
    Ok(DownloadResult { body, content_type })
}

async fn delete_s3_compatible(
    config: &StorageChannelConfig,
    object_key: &str,
) -> Result<(), StorageError> {
    let region = config.region.as_deref().unwrap_or("auto");
    let payload_sha256 = hex::encode(Sha256::digest([]));
    let url = build_s3_compatible_object_url(&config.endpoint, &config.bucket, object_key);
    let credentials = Credentials::new(
        config.access_key_id.clone(),
        config.access_key_secret.clone(),
        None,
        None,
        "licensehub",
    );
    let identity = credentials.into();
    let params = v4::SigningParams::builder()
        .identity(&identity)
        .region(region)
        .name("s3")
        .time(SystemTime::now())
        .settings(SigningSettings::default())
        .build()
        .map_err(|error| {
            StorageError::Signature(format!("failed to build signing params: {error}"))
        })?
        .into();
    let signable = SignableRequest::new(
        "DELETE",
        &url,
        [("x-amz-content-sha256", payload_sha256.as_str())].into_iter(),
        SignableBody::Bytes(&[]),
    )
    .map_err(|error| {
        StorageError::Signature(format!("failed to build signable request: {error}"))
    })?;
    let (instructions, _signature) = sign(signable, &params)
        .map_err(|error| {
            StorageError::Signature(format!("failed to sign storage request: {error}"))
        })?
        .into_parts();
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("x-amz-content-sha256"),
        HeaderValue::from_str(&payload_sha256).map_err(|error| {
            StorageError::Signature(format!("invalid payload hash header value: {error}"))
        })?,
    );
    for (name, value) in instructions.headers() {
        headers.insert(
            HeaderName::from_bytes(name.as_bytes()).map_err(|error| {
                StorageError::Signature(format!("invalid signed header name: {error}"))
            })?,
            HeaderValue::from_str(value).map_err(|error| {
                StorageError::Signature(format!("invalid signed header value: {error}"))
            })?,
        );
    }
    let response = reqwest::Client::new()
        .delete(url)
        .headers(headers)
        .send()
        .await
        .map_err(|error| StorageError::Request(error.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(StorageError::Response(format!(
            "delete failed with status {status}: {text}"
        )));
    }
    Ok(())
}

fn build_s3_compatible_object_url(endpoint: &str, bucket: &str, object_key: &str) -> String {
    let endpoint = endpoint.trim_end_matches('/');
    let encoded_bucket = path_segment(bucket.trim());
    let encoded_key = object_key
        .split('/')
        .map(path_segment)
        .collect::<Vec<_>>()
        .join("/");

    if endpoint
        .rsplit('/')
        .next()
        .is_some_and(|segment| segment == bucket.trim() || segment == encoded_bucket)
    {
        format!("{endpoint}/{encoded_key}")
    } else {
        format!("{endpoint}/{encoded_bucket}/{encoded_key}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_object_url_appends_bucket_for_root_endpoint() {
        assert_eq!(
            build_s3_compatible_object_url(
                "https://account.r2.cloudflarestorage.com",
                "apphub",
                "apps/NAE2W1U7444J/latest.json"
            ),
            "https://account.r2.cloudflarestorage.com/apphub/apps/NAE2W1U7444J/latest.json"
        );
    }

    #[test]
    fn r2_object_url_does_not_duplicate_bucket_path() {
        assert_eq!(
            build_s3_compatible_object_url(
                "https://account.r2.cloudflarestorage.com/apphub/",
                "apphub",
                "apps/NAE2W1U7444J/latest.json"
            ),
            "https://account.r2.cloudflarestorage.com/apphub/apps/NAE2W1U7444J/latest.json"
        );
    }
}
