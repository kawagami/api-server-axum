//! 把**不是 handler 產生**的錯誤回應換成全站統一形狀。
//!
//! `crate::extract` 收掉的是 extractor 的 rejection，但還有三種錯誤根本輪不到 extractor：
//! - **408** —— `TimeoutLayer`（60 秒，見 `routes.rs`）逾時，回**空 body**
//! - **413** —— `RequestBodyLimitLayer` 直接回純文字 `length limit exceeded`
//! - **405** —— Router 對不上 method，回**空 body** + `Allow` header
//!
//! 三者都不經過 `AppError`：body 形狀不一致、body 內沒有 `request_id`、也不落 log。
//! 413 尤其該補 —— 上傳失敗是這個站已知的痛點（nginx `client_max_body_size` 與這裡的
//! 10MB 必須相等，見 `routes.rs`），而「上傳到一半斷掉」正是最難查的那種回報。
//!
//! ⚠️ **判斷條件刻意是「4xx/5xx 且 content-type 不是 JSON」**，不是列舉狀態碼：
//! 列舉的話，哪天多一個 layer 回新的狀態碼就又靜默漏掉一種。`AppError` 產生的回應
//! 一律是 `application/json`，會原樣放行。

use crate::errors::{normalize_error_response, MAX_ERROR_BODY};
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn error_shape(req: Request, next: Next) -> Response {
    let response = next.run(req).await;
    let status = response.status();

    if !status.is_client_error() && !status.is_server_error() {
        return response;
    }

    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if is_json {
        return response;
    }

    let (parts, body) = response.into_parts();
    // 錯誤 body 都很短；讀不回來（串流到一半失敗之類）就用狀態碼的標準說明頂上
    let text = match axum::body::to_bytes(body, MAX_ERROR_BODY).await {
        Ok(bytes) => String::from_utf8_lossy(&bytes).trim().to_string(),
        Err(_) => String::new(),
    };
    let message = if text.is_empty() {
        status.canonical_reason().unwrap_or("request failed").to_string()
    } else {
        text
    };

    let mut normalized = normalize_error_response(status, message).into_response();
    // 保留原回應的 header（405 的 `Allow` 是協定要求的，掉了就不合規）
    for (name, value) in parts.headers.iter() {
        if name != header::CONTENT_TYPE && name != header::CONTENT_LENGTH {
            normalized.headers_mut().insert(name, value.clone());
        }
    }
    normalized
}
