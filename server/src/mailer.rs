use crate::core::my_error::AppError;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};

#[derive(Debug, Clone)]
pub struct EmailServiceConfig {
    pub mode: String,
    pub from: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_tls_mode: String,
}

pub async fn send_email_code(
    config: &EmailServiceConfig,
    email: &str,
    code: &str,
    expires_minutes: i64,
) -> Result<(), AppError> {
    if config.mode == "log" {
        tracing::info!(
            target: "licensehub::development_email",
            email = %mask_email(email),
            code = %code,
            expires_minutes,
            "development email verification code"
        );
        return Ok(());
    }

    let from = parse_mailbox(&config.from, "email_from")?;
    let to = parse_mailbox(email, "email")?;
    let credentials = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());
    let builder = match config.smtp_tls_mode.as_str() {
        "tls" => AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host),
        "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            &config.smtp_host,
        )),
        _ => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host),
    }
    .map_err(|error| AppError::business_logic("EMAIL_CONFIG_INVALID", error.to_string()))?;
    let transport = builder
        .port(config.smtp_port)
        .credentials(credentials)
        .timeout(Some(std::time::Duration::from_secs(15)))
        .build();

    let subject = "LicenseHub 注册验证码";
    let text = format!(
        "你的 LicenseHub 注册验证码是：{code}\n\n验证码 {expires_minutes} 分钟内有效。如非本人操作，请忽略此邮件。"
    );
    let html = format!(
        r#"<!doctype html><html><body style="font-family:Arial,sans-serif;color:#1f2937;line-height:1.6"><p>你的 LicenseHub 注册验证码是：</p><p style="font-size:30px;font-weight:700;letter-spacing:6px">{code}</p><p>验证码 {expires_minutes} 分钟内有效。如非本人操作，请忽略此邮件。</p></body></html>"#
    );
    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html),
                ),
        )
        .map_err(|error| AppError::business_logic("EMAIL_BUILD_FAILED", error.to_string()))?;
    transport
        .send(message)
        .await
        .map_err(|error| AppError::ExternalService {
            service: "SMTP".to_string(),
            error: error.to_string(),
        })?;
    Ok(())
}

fn parse_mailbox(value: &str, field: &str) -> Result<Mailbox, AppError> {
    value.parse::<Mailbox>().map_err(|error| {
        AppError::business_logic("EMAIL_CONFIG_INVALID", format!("{field}: {error}"))
    })
}

pub fn mask_email(email: &str) -> String {
    let Some((local, domain)) = email.split_once('@') else {
        return "***".to_string();
    };
    let first = local.chars().next().unwrap_or('*');
    format!("{first}***@{domain}")
}
