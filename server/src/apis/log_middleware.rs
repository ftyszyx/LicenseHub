use bytes::Bytes;
use salvo::http::ReqBody;
use salvo::http::header;
use salvo::prelude::*;
use std::collections::VecDeque;

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

    let req_body_bytes = match req.payload_with_max_size(64 * 1024).await {
        Ok(b) => {
            let cloned = b.clone();
            req.replace_body(ReqBody::Once(cloned.clone()));
            Some(cloned)
        }
        Err(_) => None,
    };

    match req_body_bytes {
        Some(b) if !b.is_empty() => {
            let (body_str, truncated) = format_body_preview(&b);
            tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, truncated, body = %body_str);
        }
        _ => {
            tracing::info!(target: "app_server::request", %method, %path, %query, content_type = %req_content_type, body = "<empty>");
        }
    }

    ctrl.call_next(req, depot, res).await;
    let status = res.status_code.map(|s| s.as_u16()).unwrap_or(0);

    let content_type = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = res.take_body();
    match body {
        salvo::http::ResBody::None => {
            tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, body = "<empty>");
            res.body(salvo::http::ResBody::None);
        }
        salvo::http::ResBody::Once(bytes) => {
            let (body_str, truncated) = format_body_preview(&bytes);
            tracing::info!(target: "app_server::response", %method, %path, %status, %content_type, truncated, body = %body_str);
            res.body(bytes);
        }
        salvo::http::ResBody::Chunks(chunks) => {
            let bytes = join_chunks(chunks);
            let (body_str, truncated) = format_body_preview(&bytes);
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

fn format_body_preview(bytes: &Bytes) -> (String, bool) {
    const MAX_LEN: usize = 4096;
    let truncated = bytes.len() > MAX_LEN;
    let slice = if truncated {
        &bytes[..MAX_LEN]
    } else {
        &bytes[..]
    };
    let s = String::from_utf8_lossy(slice).to_string();
    (s, truncated)
}
