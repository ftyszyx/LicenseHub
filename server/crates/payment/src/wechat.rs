use crate::types::{
    CreatePaymentRequest, CreatePaymentResponse, PaymentAdapter, PaymentError, PaymentHeaders,
    PaymentNotification, PaymentStatus,
};
use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand::Rng;
use rand::distributions::Alphanumeric;
use rsa::pkcs1v15::{Signature as RsaSignature, SigningKey, VerifyingKey};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;

const PROVIDER_WECHAT: &str = "wechat";
const PAY_TYPE_WECHAT_NATIVE: &str = "wechat_native";
const WECHATPAY_MAX_CLOCK_SKEW_SECONDS: u64 = 5 * 60;

#[derive(Debug, Clone)]
pub struct WechatNativeConfig {
    pub app_id: String,
    pub mch_id: String,
    pub merchant_serial_no: String,
    pub merchant_private_key_pem: String,
    pub api_v3_key: String,
    pub wechatpay_public_key_id: String,
    pub wechatpay_public_key_pem: String,
    pub api_base_url: String,
}

impl WechatNativeConfig {
    pub fn validate(&self) -> Result<(), PaymentError> {
        for (name, value) in [
            ("app_id", self.app_id.as_str()),
            ("mch_id", self.mch_id.as_str()),
            ("merchant_serial_no", self.merchant_serial_no.as_str()),
            (
                "merchant_private_key_pem",
                self.merchant_private_key_pem.as_str(),
            ),
            ("api_v3_key", self.api_v3_key.as_str()),
            (
                "wechatpay_public_key_id",
                self.wechatpay_public_key_id.as_str(),
            ),
            (
                "wechatpay_public_key_pem",
                self.wechatpay_public_key_pem.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                return Err(PaymentError::Config(format!("{} must be set", name)));
            }
        }
        if self.api_v3_key.chars().count() != 32 {
            return Err(PaymentError::Config(
                "api_v3_key must be a 32-character string".to_string(),
            ));
        }
        if !self
            .wechatpay_public_key_id
            .trim()
            .starts_with("PUB_KEY_ID_")
        {
            return Err(PaymentError::Config(
                "wechatpay_public_key_id must start with PUB_KEY_ID_".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct WechatNativeAdapter {
    config: WechatNativeConfig,
    merchant_private_key: RsaPrivateKey,
    wechatpay_public_key: RsaPublicKey,
    client: reqwest::Client,
}

impl WechatNativeAdapter {
    pub fn new(mut config: WechatNativeConfig) -> Result<Self, PaymentError> {
        config.api_base_url = config.api_base_url.trim_end_matches('/').to_string();
        config.merchant_private_key_pem = normalize_pem(&config.merchant_private_key_pem);
        config.wechatpay_public_key_id = config.wechatpay_public_key_id.trim().to_string();
        config.wechatpay_public_key_pem = normalize_pem(&config.wechatpay_public_key_pem);
        config.validate()?;
        let merchant_private_key = RsaPrivateKey::from_pkcs8_pem(&config.merchant_private_key_pem)
            .map_err(|error| {
                PaymentError::Config(format!("invalid WeChat merchant private key: {}", error))
            })?;
        let wechatpay_public_key = parse_public_key(&config.wechatpay_public_key_pem)?;
        Ok(Self {
            config,
            merchant_private_key,
            wechatpay_public_key,
            client: reqwest::Client::new(),
        })
    }

    fn authorization(
        &self,
        method: &str,
        canonical_url: &str,
        body: &str,
    ) -> Result<String, PaymentError> {
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let nonce = random_nonce();
        let message = format!("{method}\n{canonical_url}\n{timestamp}\n{nonce}\n{body}\n");
        let signing_key = SigningKey::<Sha256>::new(self.merchant_private_key.clone());
        let signature = signing_key.sign_with_rng(&mut rand::thread_rng(), message.as_bytes());
        Ok(format!(
            "WECHATPAY2-SHA256-RSA2048 mchid=\"{}\",nonce_str=\"{}\",timestamp=\"{}\",serial_no=\"{}\",signature=\"{}\"",
            self.config.mch_id,
            nonce,
            timestamp,
            self.config.merchant_serial_no,
            BASE64.encode(signature.to_vec())
        ))
    }

    fn verify_wechatpay_signature(
        &self,
        headers: &PaymentHeaders,
        body: &[u8],
    ) -> Result<(), PaymentError> {
        let wechatpay_serial = get_header(headers, "wechatpay-serial")?;
        if wechatpay_serial != self.config.wechatpay_public_key_id {
            return Err(PaymentError::Signature(format!(
                "WeChat Pay public key id mismatch: expected {}, got {}",
                self.config.wechatpay_public_key_id, wechatpay_serial
            )));
        }
        let timestamp = get_header(headers, "wechatpay-timestamp")?;
        validate_wechatpay_timestamp(timestamp)?;
        let nonce = get_header(headers, "wechatpay-nonce")?;
        let signature = get_header(headers, "wechatpay-signature")?;
        let body = std::str::from_utf8(body).map_err(|error| {
            PaymentError::Notification(format!("notification body is not utf-8: {}", error))
        })?;
        let message = format!("{timestamp}\n{nonce}\n{body}\n");
        let signature_bytes = BASE64.decode(signature).map_err(|error| {
            PaymentError::Signature(format!("invalid WeChat Pay signature: {}", error))
        })?;
        let signature = RsaSignature::try_from(signature_bytes.as_slice()).map_err(|error| {
            PaymentError::Signature(format!("invalid WeChat Pay signature: {}", error))
        })?;
        let verifying_key = VerifyingKey::<Sha256>::new(self.wechatpay_public_key.clone());
        verifying_key
            .verify(message.as_bytes(), &signature)
            .map_err(|error| {
                PaymentError::Signature(format!(
                    "WeChat Pay signature verification failed: {}",
                    error
                ))
            })
    }

    fn decrypt_resource(&self, resource: &WechatNotifyResource) -> Result<String, PaymentError> {
        if resource.algorithm != "AEAD_AES_256_GCM" {
            return Err(PaymentError::Notification(format!(
                "unsupported WeChat resource algorithm: {}",
                resource.algorithm
            )));
        }
        let ciphertext = BASE64.decode(&resource.ciphertext).map_err(|error| {
            PaymentError::Notification(format!("invalid WeChat ciphertext: {}", error))
        })?;
        let cipher =
            Aes256Gcm::new_from_slice(self.config.api_v3_key.as_bytes()).map_err(|_| {
                PaymentError::Config("api_v3_key must be a 32-byte AES key".to_string())
            })?;
        let aad = resource.associated_data.as_deref().unwrap_or("").as_bytes();
        let plaintext = cipher
            .decrypt(
                Nonce::from_slice(resource.nonce.as_bytes()),
                Payload {
                    msg: &ciphertext,
                    aad,
                },
            )
            .map_err(|_| {
                PaymentError::Notification("failed to decrypt WeChat notification".to_string())
            })?;
        String::from_utf8(plaintext).map_err(|error| {
            PaymentError::Notification(format!(
                "decrypted WeChat notification is invalid: {}",
                error
            ))
        })
    }
}

#[async_trait::async_trait]
impl PaymentAdapter for WechatNativeAdapter {
    fn provider(&self) -> &'static str {
        PROVIDER_WECHAT
    }

    fn pay_type(&self) -> &'static str {
        PAY_TYPE_WECHAT_NATIVE
    }

    fn label(&self) -> &'static str {
        "微信支付"
    }

    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        let path = "/v3/pay/transactions/native";
        let body = WechatNativeOrderRequest {
            appid: self.config.app_id.clone(),
            mchid: self.config.mch_id.clone(),
            description: request.subject,
            out_trade_no: request.out_trade_no,
            notify_url: request.notify_url,
            amount: WechatAmount {
                total: request.amount_cents,
                currency: Some("CNY".to_string()),
            },
            attach: request.attach,
            scene_info: request
                .client_ip
                .map(|payer_client_ip| WechatSceneInfo { payer_client_ip }),
        };
        let body_text = serde_json::to_string(&body).map_err(|error| {
            PaymentError::Request(format!("failed to serialize WeChat order: {}", error))
        })?;
        let authorization = self.authorization("POST", path, &body_text)?;
        let response = self
            .client
            .post(format!("{}{}", self.config.api_base_url, path))
            .header(reqwest::header::ACCEPT, "*/*")
            .header(reqwest::header::USER_AGENT, "LicenseHub/1.0")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, authorization)
            .header(
                "Wechatpay-Serial",
                self.config.wechatpay_public_key_id.as_str(),
            )
            .body(body_text)
            .send()
            .await
            .map_err(|error| PaymentError::Request(error.to_string()))?;
        let status = response.status();
        let response_headers = payment_headers_from_reqwest(response.headers());
        let response_text = response
            .text()
            .await
            .map_err(|error| PaymentError::Response(error.to_string()))?;
        if !status.is_success() {
            return Err(PaymentError::Response(format!(
                "WeChat Native order failed with status {}: {}",
                status, response_text
            )));
        }
        self.verify_wechatpay_signature(&response_headers, response_text.as_bytes())?;
        let parsed: WechatNativeOrderResponse =
            serde_json::from_str(&response_text).map_err(|error| {
                PaymentError::Response(format!("invalid WeChat Native order response: {}", error))
            })?;
        Ok(CreatePaymentResponse {
            provider: PROVIDER_WECHAT.to_string(),
            pay_type: PAY_TYPE_WECHAT_NATIVE.to_string(),
            provider_trade_no: None,
            pay_url: None,
            qr_code: Some(parsed.code_url.clone()),
            url_scheme: None,
            raw_payload: json!({
                "code_url": parsed.code_url,
            }),
        })
    }

    async fn parse_notification(
        &self,
        headers: &PaymentHeaders,
        body: &[u8],
    ) -> Result<PaymentNotification, PaymentError> {
        self.verify_wechatpay_signature(headers, body)?;
        let notify: WechatNotifyBody = serde_json::from_slice(body).map_err(|error| {
            PaymentError::Notification(format!("invalid WeChat notification: {}", error))
        })?;
        let decrypted = self.decrypt_resource(&notify.resource)?;
        let transaction: WechatTransaction = serde_json::from_str(&decrypted).map_err(|error| {
            PaymentError::Notification(format!("invalid WeChat transaction payload: {}", error))
        })?;
        let status = wechat_payment_status(&transaction.trade_state);
        Ok(PaymentNotification {
            provider: PROVIDER_WECHAT.to_string(),
            pay_type: PAY_TYPE_WECHAT_NATIVE.to_string(),
            out_trade_no: transaction.out_trade_no.clone(),
            provider_trade_no: transaction.transaction_id.clone(),
            amount_cents: transaction.amount.total,
            status,
            raw_payload: json!({
                "notification": notify,
                "transaction": transaction,
            }),
        })
    }

    async fn query_payment(
        &self,
        out_trade_no: &str,
    ) -> Result<Option<PaymentNotification>, PaymentError> {
        let path = format!("/v3/pay/transactions/out-trade-no/{out_trade_no}");
        let canonical_url = format!("{}?mchid={}", path, self.config.mch_id);
        let authorization = self.authorization("GET", &canonical_url, "")?;
        let response = self
            .client
            .get(format!("{}{}", self.config.api_base_url, canonical_url))
            .header(reqwest::header::ACCEPT, "*/*")
            .header(reqwest::header::USER_AGENT, "LicenseHub/1.0")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, authorization)
            .header(
                "Wechatpay-Serial",
                self.config.wechatpay_public_key_id.as_str(),
            )
            .send()
            .await
            .map_err(|error| PaymentError::Request(error.to_string()))?;
        let status = response.status();
        let response_headers = payment_headers_from_reqwest(response.headers());
        let response_text = response
            .text()
            .await
            .map_err(|error| PaymentError::Response(error.to_string()))?;
        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !status.is_success() {
            return Err(PaymentError::Response(format!(
                "WeChat order query failed with status {}: {}",
                status, response_text
            )));
        }
        self.verify_wechatpay_signature(&response_headers, response_text.as_bytes())?;
        let transaction: WechatTransaction =
            serde_json::from_str(&response_text).map_err(|error| {
                PaymentError::Response(format!("invalid WeChat order query response: {}", error))
            })?;
        Ok(Some(PaymentNotification {
            provider: PROVIDER_WECHAT.to_string(),
            pay_type: PAY_TYPE_WECHAT_NATIVE.to_string(),
            out_trade_no: transaction.out_trade_no.clone(),
            provider_trade_no: transaction.transaction_id.clone(),
            amount_cents: transaction.amount.total,
            status: wechat_payment_status(&transaction.trade_state),
            raw_payload: json!({
                "query": transaction,
            }),
        }))
    }
}

#[derive(Debug, Serialize)]
struct WechatNativeOrderRequest {
    appid: String,
    mchid: String,
    description: String,
    out_trade_no: String,
    notify_url: String,
    amount: WechatAmount,
    #[serde(skip_serializing_if = "Option::is_none")]
    attach: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scene_info: Option<WechatSceneInfo>,
}

#[derive(Debug, Serialize)]
struct WechatSceneInfo {
    payer_client_ip: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WechatAmount {
    total: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WechatNativeOrderResponse {
    code_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WechatNotifyBody {
    id: String,
    create_time: String,
    event_type: String,
    resource_type: String,
    summary: String,
    resource: WechatNotifyResource,
}

#[derive(Debug, Deserialize, Serialize)]
struct WechatNotifyResource {
    algorithm: String,
    ciphertext: String,
    nonce: String,
    #[serde(default)]
    associated_data: Option<String>,
    #[serde(default)]
    original_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WechatTransaction {
    appid: String,
    mchid: String,
    out_trade_no: String,
    #[serde(default)]
    transaction_id: Option<String>,
    trade_type: String,
    trade_state: String,
    amount: WechatTransactionAmount,
}

#[derive(Debug, Deserialize, Serialize)]
struct WechatTransactionAmount {
    total: i32,
    #[serde(default)]
    payer_total: Option<i32>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    payer_currency: Option<String>,
}

fn normalize_pem(value: &str) -> String {
    value.trim().replace("\\n", "\n")
}

fn random_nonce() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

fn get_header<'a>(headers: &'a PaymentHeaders, name: &str) -> Result<&'a str, PaymentError> {
    headers
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| PaymentError::Signature(format!("missing WeChat Pay header {}", name)))
}

fn payment_headers_from_reqwest(headers: &reqwest::header::HeaderMap) -> PaymentHeaders {
    let mut payment_headers = PaymentHeaders::new();
    for (name, value) in headers {
        if let Ok(value) = value.to_str() {
            payment_headers.insert(name.as_str().to_ascii_lowercase(), value.to_string());
        }
    }
    payment_headers
}

fn parse_public_key(pem: &str) -> Result<RsaPublicKey, PaymentError> {
    RsaPublicKey::from_public_key_pem(pem).map_err(|error| {
        PaymentError::Config(format!("invalid WeChat Pay public key PEM: {}", error))
    })
}

fn wechat_payment_status(trade_state: &str) -> PaymentStatus {
    match trade_state {
        "SUCCESS" => PaymentStatus::Success,
        "CLOSED" | "REVOKED" => PaymentStatus::Closed,
        "PAYERROR" => PaymentStatus::Failed,
        _ => PaymentStatus::Pending,
    }
}

fn validate_wechatpay_timestamp(timestamp: &str) -> Result<(), PaymentError> {
    let timestamp = timestamp.parse::<i64>().map_err(|error| {
        PaymentError::Signature(format!("invalid WeChat Pay timestamp: {}", error))
    })?;
    let now = chrono::Utc::now().timestamp();
    if now.abs_diff(timestamp) >= WECHATPAY_MAX_CLOCK_SKEW_SECONDS {
        return Err(PaymentError::Signature(format!(
            "WeChat Pay timestamp {} is outside the allowed 5-minute window",
            timestamp
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_current_wechatpay_timestamp() {
        let timestamp = chrono::Utc::now().timestamp().to_string();

        validate_wechatpay_timestamp(&timestamp).expect("current timestamp should be accepted");
    }

    #[test]
    fn rejects_expired_wechatpay_timestamp() {
        let timestamp =
            (chrono::Utc::now().timestamp() - WECHATPAY_MAX_CLOCK_SKEW_SECONDS as i64 - 1)
                .to_string();

        assert!(validate_wechatpay_timestamp(&timestamp).is_err());
    }
}
