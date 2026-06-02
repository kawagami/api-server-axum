use crate::{errors::AppError, state::AppState};
use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use redis::AsyncCommands;
use std::net::SocketAddr;

const MAX_REQUESTS: i64 = 20;
const WINDOW_SECS: i64 = 60;

pub async fn tools_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let ip = req
        .headers()
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    let key = format!("rate_limit:tools:{}", ip);
    let mut conn = state.get_redis_conn().await?;

    let count: i64 = conn.incr(&key, 1).await?;
    if count == 1 {
        let _: () = conn.expire(&key, WINDOW_SECS).await?;
    }

    if count > MAX_REQUESTS {
        return Ok((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "code": 429,
                "message": "請求過於頻繁，請稍後再試"
            })),
        )
            .into_response());
    }

    Ok(next.run(req).await)
}
