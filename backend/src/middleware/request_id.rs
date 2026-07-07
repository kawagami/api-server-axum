use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use uuid::Uuid;

/// request 專屬追蹤 id。放進 request extensions 供 TraceLayer 的 span 讀取，
/// 同時透過 task-local 讓 `AppError::into_response` 能塞進錯誤 body。
#[derive(Clone)]
pub struct RequestId(pub String);

tokio::task_local! {
    static REQUEST_ID: String;
}

/// 取得目前 request 的 id（僅在 `request_id` middleware 的 scope 內有值）。
pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(|id| id.clone()).ok()
}

const HEADER: &str = "x-request-id";

/// 產生（或沿用上游帶入的）request id，寫進 extensions + task-local + response header。
/// 掛在最外層：後續的 TraceLayer span 與所有 handler log 都會帶上此 id，錯誤可回溯。
pub async fn request_id(mut req: Request, next: Next) -> Response {
    let id = req
        .headers()
        .get(HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.extensions_mut().insert(RequestId(id.clone()));

    let header_value = HeaderValue::from_str(&id).ok();
    let mut res = REQUEST_ID.scope(id, next.run(req)).await;

    if let Some(value) = header_value {
        res.headers_mut().insert(HEADER, value);
    }
    res
}
