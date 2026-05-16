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

#[derive(Debug, Clone)]
pub struct WechatNativeConfig {
    pub app_id: String,
    pub mch_id: String,
    pub merchant_serial_no: String,
    pub merchant_private_key_pem: String,
    pub api_v3_key: String,
    pub platform_public_key_pem: Option<String>,
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
        ] {
            if value.trim().is_empty() {
                return Err(PaymentError::Config(format!("{} must be set", name)));
            }
        }
        if self.api_v3_key.as_bytes().len() != 32 {
            return Err(PaymentError::Config(
                "api_v3_key must be 32 bytes".to_string(),
            ));
        }
        Ok(())
    }
}

pub struct WechatNativeAdapter {
    config: WechatNativeConfig,
    merchant_private_key: RsaPrivateKey,
    platform_public_key: Option<RsaPublicKey>,
    client: reqwest::Client,
}

impl WechatNativeAdapter {
    pub fn new(mut config: WechatNativeConfig) -> Result<Self, PaymentError> {
        config.api_base_url = config.api_base_url.trim_end_matches('/').to_string();
        config.merchant_private_key_pem = normalize_pem(&config.merchant_private_key_pem);
        config.platform_public_key_pem = config
            .platform_public_key_pem
            .map(|pem| normalize_pem(&pem));
        config.validate()?;
        let merchant_private_key = RsaPrivateKey::from_pkcs8_pem(&config.merchant_private_key_pem)
            .map_err(|error| {
                PaymentError::Config(format!("invalid WeChat merchant private key: {}", error))
            })?;
        let platform_public_key = config
            .platform_public_key_pem
            .as_deref()
            .map(parse_public_key)
            .transpose()?;
        Ok(Self {
            config,
            merchant_private_key,
            platform_public_key,
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

    fn verify_notification_signature(
        &self,
        headers: &PaymentHeaders,
        body: &[u8],
    ) -> Result<(), PaymentError> {
        let platform_public_key = self.platform_public_key.as_ref().ok_or_else(|| {
            PaymentError::Config(
                "platform_public_key_pem is required for notifications".to_string(),
            )
        })?;
        let timestamp = get_header(headers, "wechatpay-timestamp")?;
        let nonce = get_header(headers, "wechatpay-nonce")?;
        let signature = get_header(headers, "wechatpay-signature")?;
        let body = std::str::from_utf8(body).map_err(|error| {
            PaymentError::Notification(format!("notification body is not utf-8: {}", error))
        })?;
        let message = format!("{timestamp}\n{nonce}\n{body}\n");
        let signature_bytes = BASE64.decode(signature).map_err(|error| {
            PaymentError::Signature(format!("invalid WeChat notification signature: {}", error))
        })?;
        let signature = RsaSignature::try_from(signature_bytes.as_slice()).map_err(|error| {
            PaymentError::Signature(format!("invalid WeChat notification signature: {}", error))
        })?;
        let verifying_key = VerifyingKey::<Sha256>::new(platform_public_key.clone());
        verifying_key
            .verify(message.as_bytes(), &signature)
            .map_err(|error| {
                PaymentError::Signature(format!(
                    "WeChat notification signature verification failed: {}",
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
        let cipher = Aes256Gcm::new_from_slice(self.config.api_v3_key.as_bytes())
            .map_err(|_| PaymentError::Config("api_v3_key must be 32 bytes".to_string()))?;
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
            .header(reqwest::header::ACCEPT, "application/json")
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::AUTHORIZATION, authorization)
            .body(body_text)
            .send()
            .await
            .map_err(|error| PaymentError::Request(error.to_string()))?;
        let status = response.status();
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
        self.verify_notification_signature(headers, body)?;
        let notify: WechatNotifyBody = serde_json::from_slice(body).map_err(|error| {
            PaymentError::Notification(format!("invalid WeChat notification: {}", error))
        })?;
        let decrypted = self.decrypt_resource(&notify.resource)?;
        let transaction: WechatTransaction = serde_json::from_str(&decrypted).map_err(|error| {
            PaymentError::Notification(format!("invalid WeChat transaction payload: {}", error))
        })?;
        let status = match transaction.trade_state.as_str() {
            "SUCCESS" => PaymentStatus::Success,
            "CLOSED" | "REVOKED" => PaymentStatus::Closed,
            "PAYERROR" => PaymentStatus::Failed,
            _ => PaymentStatus::Pending,
        };
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
        .ok_or_else(|| PaymentError::Notification(format!("missing header {}", name)))
}

fn parse_public_key(pem: &str) -> Result<RsaPublicKey, PaymentError> {
    RsaPublicKey::from_public_key_pem(pem).map_err(|error| {
        PaymentError::Config(format!("invalid WeChat platform public key PEM: {}", error))
    })
}
