use crate::types::{
    CreatePaymentRequest, CreatePaymentResponse, PaymentAdapter, PaymentError, PaymentHeaders,
    PaymentNotification, PaymentStatus,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rsa::pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey};
use rsa::pkcs1v15::{Signature as RsaSignature, SigningKey, VerifyingKey};
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey};
use rsa::signature::{SignatureEncoding, Signer, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::Serialize;
use serde_json::json;
use sha2::Sha256;
use std::collections::BTreeMap;
use url::form_urlencoded;

const PROVIDER_ALIPAY: &str = "alipay";
const PAY_TYPE_ALIPAY: &str = "alipay";
const METHOD_PAGE_PAY: &str = "alipay.trade.page.pay";
const PRODUCT_CODE_PAGE_PAY: &str = "FAST_INSTANT_TRADE_PAY";
const SIGN_TYPE_RSA2: &str = "RSA2";

#[derive(Debug, Clone)]
pub struct AlipayPageConfig {
    pub app_id: String,
    pub app_private_key_pem: String,
    pub alipay_public_key_pem: String,
    pub gateway_url: String,
    pub seller_id: Option<String>,
}

impl AlipayPageConfig {
    pub fn validate(&self) -> Result<(), PaymentError> {
        for (name, value) in [
            ("app_id", self.app_id.as_str()),
            ("app_private_key_pem", self.app_private_key_pem.as_str()),
            ("alipay_public_key_pem", self.alipay_public_key_pem.as_str()),
            ("gateway_url", self.gateway_url.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(PaymentError::Config(format!("{} must be set", name)));
            }
        }
        Ok(())
    }
}

pub struct AlipayPageAdapter {
    config: AlipayPageConfig,
    app_private_key: RsaPrivateKey,
    alipay_public_key: RsaPublicKey,
}

impl AlipayPageAdapter {
    pub fn new(mut config: AlipayPageConfig) -> Result<Self, PaymentError> {
        config.gateway_url = config.gateway_url.trim().to_string();
        config.app_private_key_pem = normalize_private_key(&config.app_private_key_pem);
        config.alipay_public_key_pem = normalize_public_key(&config.alipay_public_key_pem);
        config.seller_id = config
            .seller_id
            .and_then(|value| non_empty(value.trim().to_string()));
        config.validate()?;
        let app_private_key = parse_private_key(&config.app_private_key_pem)?;
        let alipay_public_key = parse_public_key(&config.alipay_public_key_pem)?;
        Ok(Self {
            config,
            app_private_key,
            alipay_public_key,
        })
    }

    fn sign(&self, params: &BTreeMap<String, String>) -> String {
        let content = sign_content(params);
        let signing_key = SigningKey::<Sha256>::new(self.app_private_key.clone());
        let signature = signing_key.sign(content.as_bytes());
        BASE64.encode(signature.to_vec())
    }

    fn verify_notification(&self, params: &BTreeMap<String, String>) -> Result<(), PaymentError> {
        let sign_type = get_param(params, "sign_type")?;
        if sign_type != SIGN_TYPE_RSA2 {
            return Err(PaymentError::Signature(format!(
                "unsupported Alipay sign_type: {}",
                sign_type
            )));
        }
        let sign = get_param(params, "sign")?;
        let content = notification_sign_content(params);
        let signature_bytes = BASE64.decode(sign).map_err(|error| {
            PaymentError::Signature(format!("invalid Alipay notification signature: {}", error))
        })?;
        let signature = RsaSignature::try_from(signature_bytes.as_slice()).map_err(|error| {
            PaymentError::Signature(format!("invalid Alipay notification signature: {}", error))
        })?;
        let verifying_key = VerifyingKey::<Sha256>::new(self.alipay_public_key.clone());
        verifying_key
            .verify(content.as_bytes(), &signature)
            .map_err(|error| {
                PaymentError::Signature(format!(
                    "Alipay notification signature verification failed: {}",
                    error
                ))
            })
    }

    fn validate_notification_params(
        &self,
        params: &BTreeMap<String, String>,
    ) -> Result<(), PaymentError> {
        let app_id = get_param(params, "app_id")?;
        if app_id != self.config.app_id {
            return Err(PaymentError::Notification(format!(
                "Alipay app_id mismatch: {}",
                app_id
            )));
        }
        if let Some(expected_seller_id) = &self.config.seller_id {
            let seller_id = get_param(params, "seller_id")?;
            if seller_id != expected_seller_id {
                return Err(PaymentError::Notification(format!(
                    "Alipay seller_id mismatch: {}",
                    seller_id
                )));
            }
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl PaymentAdapter for AlipayPageAdapter {
    fn provider(&self) -> &'static str {
        PROVIDER_ALIPAY
    }

    fn pay_type(&self) -> &'static str {
        PAY_TYPE_ALIPAY
    }

    fn label(&self) -> &'static str {
        "Alipay"
    }

    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError> {
        if request.amount_cents <= 0 {
            return Err(PaymentError::Request(
                "Alipay total_amount must be greater than 0".to_string(),
            ));
        }

        let biz_content = AlipayPagePayBizContent {
            out_trade_no: request.out_trade_no.clone(),
            total_amount: cents_to_amount(request.amount_cents),
            subject: sanitize_subject(&request.subject),
            product_code: PRODUCT_CODE_PAGE_PAY.to_string(),
            body: None,
            passback_params: request
                .attach
                .map(|value| form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>()),
            integration_type: Some("PCWEB".to_string()),
        };
        let biz_content = serde_json::to_string(&biz_content).map_err(|error| {
            PaymentError::Request(format!("failed to serialize Alipay order: {}", error))
        })?;

        let mut params = BTreeMap::new();
        params.insert("app_id".to_string(), self.config.app_id.clone());
        params.insert("method".to_string(), METHOD_PAGE_PAY.to_string());
        params.insert("format".to_string(), "json".to_string());
        params.insert("charset".to_string(), "UTF-8".to_string());
        params.insert("sign_type".to_string(), SIGN_TYPE_RSA2.to_string());
        params.insert(
            "timestamp".to_string(),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        );
        params.insert("version".to_string(), "1.0".to_string());
        params.insert("notify_url".to_string(), request.notify_url);
        if let Some(return_url) = request.return_url {
            params.insert("return_url".to_string(), return_url);
        }
        params.insert("biz_content".to_string(), biz_content);
        let sign = self.sign(&params);
        params.insert("sign".to_string(), sign);

        let pay_url = build_gateway_url(&self.config.gateway_url, &params);
        Ok(CreatePaymentResponse {
            provider: PROVIDER_ALIPAY.to_string(),
            pay_type: PAY_TYPE_ALIPAY.to_string(),
            provider_trade_no: None,
            pay_url: Some(pay_url.clone()),
            qr_code: None,
            url_scheme: None,
            raw_payload: json!({
                "gateway_url": self.config.gateway_url,
                "method": METHOD_PAGE_PAY,
                "pay_url": pay_url,
                "params": params,
            }),
        })
    }

    async fn parse_notification(
        &self,
        _headers: &PaymentHeaders,
        body: &[u8],
    ) -> Result<PaymentNotification, PaymentError> {
        let params = parse_form_body(body)?;
        self.verify_notification(&params)?;
        self.validate_notification_params(&params)?;

        let trade_status = get_param(&params, "trade_status")?;
        let status = match trade_status {
            "TRADE_SUCCESS" | "TRADE_FINISHED" => PaymentStatus::Success,
            "TRADE_CLOSED" => PaymentStatus::Closed,
            "WAIT_BUYER_PAY" => PaymentStatus::Pending,
            _ => PaymentStatus::Failed,
        };
        Ok(PaymentNotification {
            provider: PROVIDER_ALIPAY.to_string(),
            pay_type: PAY_TYPE_ALIPAY.to_string(),
            out_trade_no: get_param(&params, "out_trade_no")?.to_string(),
            provider_trade_no: params.get("trade_no").cloned().and_then(non_empty),
            amount_cents: parse_amount_cents(get_param(&params, "total_amount")?)?,
            status,
            raw_payload: json!({
                "params": params,
            }),
        })
    }
}

#[derive(Debug, Serialize)]
struct AlipayPagePayBizContent {
    out_trade_no: String,
    total_amount: String,
    subject: String,
    product_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    passback_params: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    integration_type: Option<String>,
}

fn sign_content(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .filter(|(key, value)| key.as_str() != "sign" && !value.is_empty())
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("&")
}

fn notification_sign_content(params: &BTreeMap<String, String>) -> String {
    params
        .iter()
        .filter(|(key, value)| {
            key.as_str() != "sign" && key.as_str() != "sign_type" && !value.is_empty()
        })
        .map(|(key, value)| format!("{}={}", key, value))
        .collect::<Vec<_>>()
        .join("&")
}

fn build_gateway_url(gateway_url: &str, params: &BTreeMap<String, String>) -> String {
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    for (key, value) in params {
        serializer.append_pair(key, value);
    }
    let query = serializer.finish();
    let separator = if gateway_url.contains('?') {
        if gateway_url.ends_with('?') || gateway_url.ends_with('&') {
            ""
        } else {
            "&"
        }
    } else {
        "?"
    };
    format!("{}{}{}", gateway_url, separator, query)
}

fn parse_form_body(body: &[u8]) -> Result<BTreeMap<String, String>, PaymentError> {
    let body_text = std::str::from_utf8(body).map_err(|error| {
        PaymentError::Notification(format!("Alipay notification body is not utf-8: {}", error))
    })?;
    let params = form_urlencoded::parse(body_text.as_bytes())
        .into_owned()
        .collect::<BTreeMap<_, _>>();
    if params.is_empty() {
        return Err(PaymentError::Notification(
            "Alipay notification body is empty".to_string(),
        ));
    }
    Ok(params)
}

fn cents_to_amount(cents: i32) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

fn parse_amount_cents(value: &str) -> Result<i32, PaymentError> {
    let value = value.trim();
    if value.is_empty() || value.starts_with('-') {
        return Err(PaymentError::Notification(format!(
            "invalid Alipay total_amount: {}",
            value
        )));
    }
    let mut parts = value.split('.');
    let yuan = parts
        .next()
        .ok_or_else(|| PaymentError::Notification("missing Alipay total_amount".to_string()))?;
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some() || fraction.len() > 2 {
        return Err(PaymentError::Notification(format!(
            "invalid Alipay total_amount: {}",
            value
        )));
    }
    let yuan = yuan.parse::<i64>().map_err(|error| {
        PaymentError::Notification(format!("invalid Alipay total_amount: {}", error))
    })?;
    let mut fraction = fraction.to_string();
    while fraction.len() < 2 {
        fraction.push('0');
    }
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i64>().map_err(|error| {
            PaymentError::Notification(format!("invalid Alipay total_amount: {}", error))
        })?
    };
    let cents = yuan
        .checked_mul(100)
        .and_then(|value| value.checked_add(fraction))
        .ok_or_else(|| {
            PaymentError::Notification(format!("Alipay total_amount is too large: {}", value))
        })?;
    i32::try_from(cents).map_err(|_| {
        PaymentError::Notification(format!("Alipay total_amount is too large: {}", value))
    })
}

fn sanitize_subject(subject: &str) -> String {
    let sanitized = subject
        .chars()
        .map(|ch| match ch {
            '/' | '=' | '&' => ' ',
            _ => ch,
        })
        .collect::<String>();
    let sanitized = sanitized.trim();
    if sanitized.is_empty() {
        return "LicenseHub Order".to_string();
    }
    sanitized.chars().take(256).collect()
}

fn get_param<'a>(
    params: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, PaymentError> {
    params
        .get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| PaymentError::Notification(format!("missing Alipay parameter {}", name)))
}

fn normalize_private_key(value: &str) -> String {
    normalize_pem(value, "PRIVATE KEY")
}

fn normalize_public_key(value: &str) -> String {
    normalize_pem(value, "PUBLIC KEY")
}

fn normalize_pem(value: &str, label: &str) -> String {
    let value = value.trim().replace("\\n", "\n");
    if value.contains("-----BEGIN") {
        return value;
    }
    let compact = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<String>();
    let mut pem = format!("-----BEGIN {}-----\n", label);
    for chunk in compact.as_bytes().chunks(64) {
        pem.push_str(&String::from_utf8_lossy(chunk));
        pem.push('\n');
    }
    pem.push_str(&format!("-----END {}-----", label));
    pem
}

fn parse_private_key(pem: &str) -> Result<RsaPrivateKey, PaymentError> {
    RsaPrivateKey::from_pkcs8_pem(pem)
        .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem))
        .map_err(|error| PaymentError::Config(format!("invalid Alipay app private key: {}", error)))
}

fn parse_public_key(pem: &str) -> Result<RsaPublicKey, PaymentError> {
    RsaPublicKey::from_public_key_pem(pem)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(pem))
        .map_err(|error| PaymentError::Config(format!("invalid Alipay public key: {}", error)))
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_amounts_without_float_rounding() {
        assert_eq!(cents_to_amount(1), "0.01");
        assert_eq!(cents_to_amount(12345), "123.45");
        assert_eq!(parse_amount_cents("0.01").unwrap(), 1);
        assert_eq!(parse_amount_cents("88").unwrap(), 8800);
        assert_eq!(parse_amount_cents("88.8").unwrap(), 8880);
        assert!(parse_amount_cents("88.888").is_err());
    }

    #[test]
    fn builds_sign_content_in_ascii_key_order() {
        let mut params = BTreeMap::new();
        params.insert("method".to_string(), METHOD_PAGE_PAY.to_string());
        params.insert("app_id".to_string(), "2014072300007148".to_string());
        params.insert("sign".to_string(), "ignored".to_string());
        params.insert("empty".to_string(), String::new());

        assert_eq!(
            sign_content(&params),
            "app_id=2014072300007148&method=alipay.trade.page.pay"
        );
    }

    #[test]
    fn sanitizes_subject_for_page_pay() {
        assert_eq!(sanitize_subject("A/B=C&D"), "A B C D");
        assert_eq!(sanitize_subject(" /&= "), "LicenseHub Order");
    }
}
