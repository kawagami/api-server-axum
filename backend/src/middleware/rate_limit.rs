use crate::{errors::AppError, state::AppState};
use axum::{
    body::Body,
    extract::{connect_info::ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::net::SocketAddr;

const TOOLS_MAX_REQUESTS: i64 = 20;
// 登入爆破防護：每次失敗都燒一次 bcrypt（~百毫秒 CPU），限流門檻收緊
const AUTH_MAX_REQUESTS: i64 = 5;
// passkey 登入獨立額度：Conditional UI 每次載入登入頁就打一發 begin，不與密碼登入互搶；
// 且簽章驗證無 bcrypt CPU 面、無爆破意義，可放寬
const WEBAUTHN_MAX_REQUESTS: i64 = 10;
// 訪客留言:公開未認證的寫入端點,收緊防灌水(正常人一次就送完)
const MESSAGES_MAX_REQUESTS: i64 = 5;
// blog 留言:同為公開寫入,獨立 bucket 不與訪客留言互搶(討論串可能連續發幾則)
const COMMENTS_MAX_REQUESTS: i64 = 10;
const WINDOW_SECS: i64 = 60;

pub async fn tools_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    rate_limit(state, req, next, "tools", TOOLS_MAX_REQUESTS).await
}

pub async fn auth_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    rate_limit(state, req, next, "auth", AUTH_MAX_REQUESTS).await
}

pub async fn webauthn_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    rate_limit(state, req, next, "webauthn", WEBAUTHN_MAX_REQUESTS).await
}

pub async fn messages_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    rate_limit(state, req, next, "messages", MESSAGES_MAX_REQUESTS).await
}

pub async fn comments_rate_limit(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    rate_limit(state, req, next, "comments", COMMENTS_MAX_REQUESTS).await
}

async fn rate_limit(
    state: AppState,
    req: Request,
    next: Next,
    scope: &str,
    max_requests: i64,
) -> Result<Response<Body>, AppError> {
    let socket_ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string());

    // 只有確定流量經 Cloudflare（TRUST_CF_HEADER=true）才信任 header，否則 header 可偽造繞過 rate limit
    let ip = if state.get_config().trust_cf_header {
        req.headers()
            .get("CF-Connecting-IP")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or(socket_ip)
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        socket_ip.unwrap_or_else(|| "unknown".to_string())
    };

    let key = format!("rate_limit:{}:{}", scope, ip);
    let mut conn = state.get_redis_conn().await?;

    // 原子：INCR + 首次設 TTL 一次完成，避免 incr 成功 expire 失敗導致 key 無 TTL 永久封鎖
    let count: i64 = redis::Script::new(
        "local c = redis.call('INCR', KEYS[1]) \
         if c == 1 then redis.call('EXPIRE', KEYS[1], ARGV[1]) end \
         return c",
    )
    .key(&key)
    .arg(WINDOW_SECS)
    .invoke_async(&mut *conn)
    .await?;

    if count > max_requests {
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
