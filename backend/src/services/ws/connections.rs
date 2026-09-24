use crate::{
    errors::{AppError, RequestError, SystemError},
    state::{AppState, DisplayTrackedConnection},
};
use axum::extract::ws::Message;
use futures::sink::SinkExt;
use std::{net::SocketAddr, time::SystemTime};

/// 連線時間對外一律用固定寬度的 ISO-8601 毫秒 UTC 字串
pub(super) fn to_iso(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// 目前所有 WS 連線（新連線在前）。呼叫端須先確認有 `ws:read` 權限。
pub async fn list_connections(state: &AppState) -> Vec<DisplayTrackedConnection> {
    let mut result: Vec<DisplayTrackedConnection> = {
        let connections = state.get_connections().lock().await;
        connections
            .iter()
            .map(|(addr, info)| DisplayTrackedConnection {
                addr: addr.to_string(),
                connected_at: to_iso(info.connected_at),
                user_email: info.user_email.clone(),
                real_ip: info.real_ip.clone(),
                user_agent: info.user_agent.clone(),
            })
            .collect()
    };

    // 新連線在前。HashMap 迭代順序不保證穩定，不排序的話前端每次輪詢列順序都會跳。
    // 同毫秒連上的用 addr 破平手，確保順序完全確定
    result.sort_by(|a, b| {
        b.connected_at
            .cmp(&a.connected_at)
            .then_with(|| a.addr.cmp(&b.addr))
    });

    result
}

/// admin 點對點直送一則 `admin_message`。呼叫端須先確認有 `ws:read` 權限。
///
/// 失敗一律回錯誤（route 轉成非 2xx）。舊版對「位址格式錯 / 連線不存在 / 送出失敗」都回 200
/// 加一段錯誤字串，呼叫端只看 status 的話會把失敗顯示成成功。
pub async fn send_admin_message(
    state: &AppState,
    from: &str,
    addr: &str,
    message: &str,
) -> Result<(), AppError> {
    // 這兩個分支不另外記 log：錯誤本身已經回給呼叫端，也已經由 errors.rs 統一記下
    // （帶 request_id），再印一行 info 只是同一件事的第二份，而且 info 不落地 logs 表。
    let socket_addr = addr.parse::<SocketAddr>().map_err(|_| {
        RequestError::InvalidContent(format!("無效的連線位址格式：{}", addr))
    })?;

    let connections = state.get_connections().lock().await;
    let tracked_conn = connections
        .get(&socket_addr)
        .ok_or(RequestError::NotFound)?;

    // 事件名一律走 WsEvent enum，不要在這裡手寫字串 ——
    // 手寫的那份不會出現在 enum 裡，前端對照表也就跟著漏掉（admin_message 原本就是這樣走丟的）
    let payload = crate::structs::ws::envelope(
        crate::structs::ws::WsEvent::AdminMessage.as_str(),
        serde_json::json!({ "content": message, "from": from }),
    );

    let mut sender_guard = tracked_conn.sender.lock().await;
    sender_guard
        .send(Message::Text(payload.into()))
        .await
        .map_err(|e| {
            tracing::error!("Failed to send message to {}: {}", socket_addr, e);
            // 這裡不立即清理連接，讓 handle_socket 中的任務處理
            SystemError::Internal(format!("訊息送出失敗：{e}"))
        })?;

    Ok(())
}
