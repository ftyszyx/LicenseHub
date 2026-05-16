use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub type PaymentHeaders = BTreeMap<String, String>;

#[derive(Debug, Error)]
pub enum PaymentError {
    #[error("payment config error: {0}")]
    Config(String),
    #[error("payment request error: {0}")]
    Request(String),
    #[error("payment response error: {0}")]
    Response(String),
    #[error("payment signature error: {0}")]
    Signature(String),
    #[error("payment notification error: {0}")]
    Notification(String),
    #[error("unsupported payment method: {0}")]
    Unsupported(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PaymentStatus {
    Pending,
    Success,
    Failed,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub pay_type: String,
    pub label: String,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct CreatePaymentRequest {
    pub out_trade_no: String,
    pub subject: String,
    pub amount_cents: i32,
    pub notify_url: String,
    pub return_url: Option<String>,
    pub client_ip: Option<String>,
    pub attach: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentResponse {
    pub provider: String,
    pub pay_type: String,
    pub provider_trade_no: Option<String>,
    pub pay_url: Option<String>,
    pub qr_code: Option<String>,
    pub url_scheme: Option<String>,
    pub raw_payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentNotification {
    pub provider: String,
    pub pay_type: String,
    pub out_trade_no: String,
    pub provider_trade_no: Option<String>,
    pub amount_cents: i32,
    pub status: PaymentStatus,
    pub raw_payload: serde_json::Value,
}

#[async_trait]
pub trait PaymentAdapter: Send + Sync {
    fn provider(&self) -> &'static str;
    fn pay_type(&self) -> &'static str;
    fn label(&self) -> &'static str;

    async fn create_payment(
        &self,
        request: CreatePaymentRequest,
    ) -> Result<CreatePaymentResponse, PaymentError>;

    async fn parse_notification(
        &self,
        headers: &PaymentHeaders,
        body: &[u8],
    ) -> Result<PaymentNotification, PaymentError>;
}
