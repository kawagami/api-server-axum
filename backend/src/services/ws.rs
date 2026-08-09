//! WS 連線票（一次性 ticket）。
//!
//! JWT **不走** WS URL query —— query 會進 access log（`routes.rs` 的 request span），
//! 所以 admin 身分改用「登入中換一張 30 秒有效的 UUID 票 → 握手時一次性消費」。

use crate::{errors::AppError, repositories::redis, state::AppState};

/// 發一張新票（呼叫端須先確認有 `ws:read` 權限）。
///
/// ⚠️ **門檻必須與 `GET /ws/connections` 一致**：票換來的連線會被標成 admin 身分，
/// 因而收得到 `broadcast_to_admins` 的 `user_joined` / `user_left` —— 那兩則帶
/// `real_ip` / `user_email` / `user_agent`。
pub async fn issue_ticket(state: &AppState, user_name: &str) -> Result<String, AppError> {
    let ticket = uuid::Uuid::new_v4().to_string();
    redis::set_ws_ticket(state.get_redis_pool(), &ticket, user_name).await?;
    Ok(ticket)
}

/// 消費一張票，回傳票主。
///
/// 無票 / 票失效 / Redis 掛掉一律回 `None` = **匿名連線**（前台訪客即此路徑），
/// 不是錯誤 —— 握手不該因為身分認不出來就失敗。
pub async fn consume_ticket(state: &AppState, ticket: &str) -> Option<String> {
    redis::consume_ws_ticket(state.get_redis_pool(), ticket)
        .await
        .ok()
        .flatten()
}
