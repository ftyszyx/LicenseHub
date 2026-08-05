use bytes::Bytes;
use salvo::http::ReqBody;
use salvo::http::header;
use salvo::prelude::*;
use serde_json::Value;
use std::collections::VecDeque;

const REQUEST_BODY_LIMIT: usize = 64 * 1024;
const BODY_PREVIEW_LIMIT: usize = 4096;

#[handler]
pub async fn log_response_body(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if !tracing::enabled!(tracing::Level::INFO) {
        ctrl.call_next(req, depot, res).await;
        return;
    }

    let path = req.uri().path().to_string();
    let method = req.method().to_string();
    let query = req.uri().query().unwrap_or("").to_string();
    let req_content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if is_json_content_type(&req_content_type) {
        let content_length = req
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok());
        if content_length.is_some_and(|length| length > REQUEST_BODY_LIMIT) {
            tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, body = "<json omitted: body exceeds logging limit>");
        } else {
            let req_body_bytes = match req.payload_with_max_size(REQUEST_BODY_LIMIT).await {
                Ok(bytes) => {
                    let cloned = bytes.clone();
                    req.replace_body(ReqBody::Once(cloned.clone()));
                    Some(cloned)
                }
                Err(_) => None,
            };
            match req_body_bytes {
                Some(bytes) if !bytes.is_empty() => {
                    let (body_str, truncated) = format_body_preview(&bytes, &req_content_type);
                    tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, truncated, body = %body_str);
                }
                Some(_) => {
                    tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, body = "<empty>");
                }
                None => {
                    tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, body = "<json omitted: unreadable or too large>");
                }
            }
        }
    } else {
        tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, body = "<omitted: non-JSON body>");
    }

    ctrl.call_next(req, depot, res).await;
    let status = res.status_code.map(|s| s.as_u16()).unwrap_or(0);

    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if !is_text_content_type(&content_type) {
        tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, body = "<omitted: non-text body>");
        return;
    }

    let body = res.take_body();
    match body {
        salvo::http::ResBody::None => {
            tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, body = "<empty>");
            res.body(salvo::http::ResBody::None);
        }
        salvo::http::ResBody::Once(bytes) => {
            let (body_str, truncated) = format_body_preview(&bytes, &content_type);
            tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, truncated, body = %body_str);
            res.body(bytes);
        }
        salvo::http::ResBody::Chunks(chunks) => {
            let bytes = join_chunks(chunks);
            let (body_str, truncated) = format_body_preview(&bytes, &content_type);
            tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, truncated, body = %body_str);
            res.body(bytes);
        }
        other => {
            tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, body = ?other);
            res.body(other);
        }
    }
}

fn join_chunks(chunks: VecDeque<Bytes>) -> Bytes {
    let mut buf: Vec<u8> = Vec::new();
    for c in chunks {
        buf.extend_from_slice(c.as_ref());
    }
    Bytes::from(buf)
}

fn format_body_preview(bytes: &Bytes, content_type: &str) -> (String, bool) {
    let content = if is_json_content_type(content_type) {
        match serde_json::from_slice::<Value>(bytes) {
            Ok(mut value) => {
                redact_json(&mut value);
                serde_json::to_string(&value)
                    .unwrap_or_else(|_| "<json omitted: serialization failed>".to_string())
            }
            Err(_) => "<json omitted: invalid JSON>".to_string(),
        }
    } else {
        String::from_utf8_lossy(bytes).to_string()
    };
    let content = content.as_bytes();
    let truncated = content.len() > BODY_PREVIEW_LIMIT;
    let slice = if truncated {
        &content[..BODY_PREVIEW_LIMIT]
    } else {
        content
    };
    (String::from_utf8_lossy(slice).to_string(), truncated)
}

fn is_json_content_type(content_type: &str) -> bool {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    mime == "application/json" || mime.ends_with("+json")
}

fn is_text_content_type(content_type: &str) -> bool {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    mime.starts_with("text/") || mime == "application/json" || mime.ends_with("+json")
}

fn redact_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if is_sensitive_key(key) {
                    *value = Value::String("***".to_string());
                } else {
                    redact_json(value);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                redact_json(value);
            }
        }
        _ => {}
    }
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("password")
        || key.contains("secret")
        || key.contains("token")
        || key.contains("private_key")
        || key.contains("api_key")
        || key.contains("credential")
        || matches!(
            key.as_str(),
            "account" | "alipay_account" | "real_name" | "settlement_account"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_preview_redacts_nested_sensitive_fields() {
        let body = Bytes::from_static(
            br#"{"username":"demo","password":"123456","settlement_account":{"account":"demo@example.com","real_name":"Demo"},"items":[{"access_token":"abc"}]}"#,
        );

        let (preview, truncated) = format_body_preview(&body, "application/json; charset=utf-8");

        assert!(!truncated);
        assert!(preview.contains("\"username\":\"demo\""));
        assert!(!preview.contains("123456"));
        assert!(!preview.contains("demo@example.com"));
        assert!(!preview.contains("Demo"));
        assert!(!preview.contains("abc"));
    }

    #[test]
    fn binary_and_multipart_content_types_are_not_text() {
        assert!(!is_text_content_type("image/png"));
        assert!(!is_text_content_type(
            "multipart/form-data; boundary=proof-boundary"
        ));
        assert!(is_text_content_type("application/problem+json"));
    }
}
