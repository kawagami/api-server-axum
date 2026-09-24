use super::{connections::to_iso, guard::MessageBudget};
use crate::state::{AppState, TrackedConnection};
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket},
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    net::SocketAddr,
    ops::ControlFlow,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::SystemTime,
};
use tokio::{
    sync::Mutex,
    time::{Duration, Instant},
};

// --- WebSocket Ping-Pong 設定 ---
const PING_INTERVAL_SECONDS: u64 = 30;
/// 沒收到 Pong 的容忍上限（兩個 ping 週期 + 緩衝）。
///
/// **只靠 send 失敗抓不到半開連線**：對端消失但 TCP 沒斷（拔網路、手機睡眠、NAT 逾時）時，
/// 寫入會先進 kernel buffer 而「成功」，可能要好幾分鐘才回報錯誤。期間那條連線會一直掛在
/// `connections` map（後台連線列表看得到）、遊戲桌位上（對手在等一個永遠不會來的走步）。
/// 瀏覽器的 WS 實作會自動回 Pong，所以收不到 Pong 就是真的沒人在了。
const PONG_TIMEOUT_SECONDS: u64 = PING_INTERVAL_SECONDS * 2 + 15;

/// 額度用盡後還能容忍多少則被丟棄的訊息才收線。
/// 正常客戶端不會到這裡（丟第一則時就已回一則 `rate_limited` error），
/// 會到的只有跑迴圈灌訊息的腳本 —— 那條連線留著只是白佔記憶體。
const MAX_DROPPED_MESSAGES: u32 = 200;

/// 單條 WS 連線的完整生命週期：登記進 `connections` → 收訊迴圈 + ping task → 任一邊結束就
/// 收掉另一邊並 `cleanup_connection` → 留一行 INFO 摘要。由 `routes/ws.rs` 的握手 handler 在
/// `on_upgrade` 裡呼叫（外層已掛好 `ws` span）。
pub async fn handle_socket(socket: WebSocket, who: SocketAddr, state: AppState, user_email: Option<String>, real_ip: String, user_agent: String) {
    let (sender, receiver) = socket.split();
    let sender_arc = Arc::new(Mutex::new(sender));

    let connected_at = SystemTime::now();
    let connection_info = TrackedConnection {
        connected_at,
        sender: sender_arc.clone(),
        user_email: user_email.clone(),
        real_ip: real_ip.clone(),
        user_agent: user_agent.clone(),
    };

    {
        let mut connections = state.get_connections().lock().await;
        connections.insert(who, connection_info);
    }

    // 含 IP / email 個資，只推給 admin 連線，不對匿名訪客廣播。
    // 欄位與 list_connections 的列一致，admin 頁可直接用這則事件插入新列，不必重抓。
    state.broadcast_to_admins(
        crate::structs::ws::WsEvent::UserJoined,
        serde_json::json!({
            "addr": who.to_string(),
            "real_ip": real_ip,
            "user_email": user_email,
            "connected_at": to_iso(connected_at),
            "user_agent": user_agent,
        }),
    );

    // 最後一次收到 Pong 的時間；recv_task 更新、ping_task 判逾時。
    // std Mutex：只包一個 Instant，鎖不跨 await。
    let last_pong = Arc::new(std::sync::Mutex::new(Instant::now()));

    // 收到的應用層訊息數。放在 task 外面共享，`select!` 不論由哪一邊勝出都拿得到
    // —— 連線結束摘要（見下方 info）少了這個數字就答不出「掉線前對方還在動嗎」。
    let msg_count = Arc::new(AtomicU64::new(0));

    // --- recv_task: 接收客戶端訊息 ---
    let recv_state_clone = state.clone();
    let recv_last_pong = last_pong.clone();
    let recv_msg_count = msg_count.clone();
    // 迴圈以「結束原因」收尾：那是摘要裡唯一能說明「為什麼掉線」的欄位，
    // 也是判斷「一群人同時掉線」是我方還是對端的依據。
    let mut recv_task = tokio::spawn(async move {
        let mut receiver = receiver;
        // 單條連線的收訊額度（純區域狀態，無鎖）。超量的訊息直接丟掉不解析 ——
        // 收線是最後手段：前端 ws-context 會自動重連，一超量就關等於送對方一個重連迴圈。
        let mut budget = MessageBudget::new();
        let mut dropped = 0u32;
        loop {
            let Some(msg_result) = receiver.next().await else {
                // 對端關掉 TCP，連 Close 帧都沒送（關分頁最常見的形狀）
                break "stream_end";
            };
            match msg_result {
                Ok(msg) => {
                    // 只算應用層訊息（Text/Binary）。控制帧不計 —— Pong 是我們自己每 30 秒
                    // 要來的，混進去會讓「掉線前對方還在動嗎」這個問題恆為 yes。
                    if matches!(msg, Message::Text(_) | Message::Binary(_)) {
                        recv_msg_count.fetch_add(1, Ordering::Relaxed);
                    }
                    // 存活證明只認 Pong（其餘訊息可能來自沒在讀我們 ping 的客戶端）
                    if matches!(msg, Message::Pong(_)) {
                        *recv_last_pong.lock().unwrap() = Instant::now();
                    }
                    // 只有應用層訊息計費；Ping/Pong/Close 是控制帧，不佔額度。
                    if matches!(msg, Message::Text(_) | Message::Binary(_)) && !budget.try_consume()
                    {
                        dropped += 1;
                        if dropped == 1 {
                            tracing::warn!("{who} 收訊超量，開始丟棄訊息");
                            recv_state_clone.send_to(
                                who,
                                crate::structs::ws::envelope(
                                    "error",
                                    serde_json::json!({ "reason": "rate_limited" }),
                                ),
                            );
                        }
                        if dropped >= MAX_DROPPED_MESSAGES {
                            tracing::warn!("{who} 持續灌訊息（已丟 {dropped} 則），收線");
                            break "flood";
                        }
                        continue;
                    }
                    if process_message(msg, who, &recv_state_clone).await.is_break() {
                        break "client_close";
                    }
                }
                Err(e) => {
                    // debug 不是 warn：這裡幾乎清一色是 "Connection reset without closing
                    // handshake" —— 關分頁、手機睡眠、NAT 逾時都會產生，是公開網站的常態
                    // （實測佔了 logs 表 WARN+ 的 45%）。而且斷線後下面照樣走完整清理，
                    // 沒有任何要人介入的事。真正需要注意的送出失敗另有其他 log。
                    tracing::debug!("Error receiving message from {}: {}", who, e);
                    break "recv_error";
                }
            }
        }
    });

    // --- ping_task: 主動發 Ping，並在遲遲收不到 Pong 時收掉連線 ---
    // 這個 task 結束 = 下面的 select! 收到 → abort recv_task → cleanup_connection，
    // 所以「判定死掉」只要 break 就會走完整的清理流程（含遊戲桌位斷線處理）。
    let ping_sender_clone = sender_arc.clone();
    let ping_last_pong = last_pong.clone();
    let mut ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECONDS));
        interval.tick().await; // 跳過第一次立即觸發

        loop {
            interval.tick().await;

            let silent_for = ping_last_pong.lock().unwrap().elapsed();
            if silent_for >= Duration::from_secs(PONG_TIMEOUT_SECONDS) {
                tracing::info!(
                    "{who} 已 {} 秒沒回 Pong，判定連線已死並清理",
                    silent_for.as_secs()
                );
                break "pong_timeout";
            }

            {
                let mut sender_guard = ping_sender_clone.lock().await;
                if let Err(e) = sender_guard.send(Message::Ping(Bytes::new())).await {
                    tracing::warn!("Failed to send ping to {who}: {}", e);
                    break "ping_failed";
                }
            }
        }
    });

    // --- tokio::select!: 協調所有任務 ---
    let reason = tokio::select! {
        rv_b = (&mut recv_task) => rv_b.unwrap_or_else(|e| {
            tracing::error!("Error in recv_task for {who}: {:?}", e);
            "recv_task_error"
        }),
        rv_c = (&mut ping_task) => rv_c.unwrap_or_else(|e| {
            tracing::error!("Error in ping_task for {who}: {:?}", e);
            "ping_task_error"
        }),
    };

    // 清理工作
    recv_task.abort();
    ping_task.abort();

    // 最終清理連接
    cleanup_connection(&state, who).await;

    // **INFO 不是 debug**：生產的 `RUST_LOG` 天花板是 `info`（見 `main.rs`），所以 WS
    // 這邊原本清一色的 `debug!` 在生產**根本不存在** —— EnvFilter 就擋掉了，stdout 與
    // `logs` 表兩邊都沒有，「一群人同時掉線」事後完全無跡可循。
    //
    // 每條連線只留這一行摘要（開了多久、收了幾則、為什麼結束），其餘識別欄位
    // （conn / ip / email / request_id）在 span 上，量級 = 每條連線一行。
    // 落地 `logs` 表仍需把 `log_db_level` 調到 INFO（預設 WARN 不收），但 stdout 一定有。
    // 逐則收訊/送出失敗維持 debug 不變（那是關分頁的常態，理由見上面的 recv 迴圈）。
    tracing::info!(
        reason,
        duration_secs = connected_at.elapsed().map(|d| d.as_secs()).unwrap_or(0),
        messages = msg_count.load(Ordering::Relaxed),
        "websocket 連線結束"
    );
}

/// 依信封 `game` 欄分派給對應遊戲 hub。回傳 true 表示已當作遊戲訊息處理。
async fn dispatch_game(state: &AppState, who: SocketAddr, value: &serde_json::Value) -> bool {
    let Some(game) = value.get("game").and_then(|v| v.as_str()) else {
        return false;
    };
    // instance 級功能開關：games 關閉時擋下所有遊戲訊息（watcher 照常跑，熱開關不需重啟）
    if !state
        .get_settings()
        .feature_enabled(crate::structs::features::Feature::Games)
    {
        state.send_to(
            who,
            crate::structs::ws::game_envelope(
                game,
                "error",
                serde_json::json!({ "reason": "feature_disabled" }),
            ),
        );
        return true;
    }
    match state.games().get(game) {
        Some(hub) => hub.handle(state, who, value).await,
        None => false,
    }
}

async fn process_message(msg: Message, who: SocketAddr, state: &AppState) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            // 解析統一信封 `{ game?, type, data }`，分派給對應遊戲 hub。
            // 非 JSON / 未知訊息一律忽略（不再 echo 廣播）。
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&t) {
                dispatch_game(state, who, &value).await;
            }
        }
        Message::Binary(_) => {}
        Message::Close(c) => {
            if let Some(cf) = c {
                tracing::debug!(
                    ">>> {who} sent close with code {} and reason `{}`",
                    cf.code,
                    cf.reason
                );
            } else {
                tracing::debug!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        // Pong 的存活記帳在 handle_socket 的 recv 迴圈（要更新 last_pong），這裡不重複處理
        Message::Pong(_) => {}
        // axum 會自動回 Pong
        Message::Ping(_) => {}
    }
    ControlFlow::Continue(())
}

async fn cleanup_connection(state: &AppState, who: SocketAddr) {
    // 各遊戲斷線清理：在佇列就移除；在對局就判對手勝（斷線即判敗）
    for hub in state.games().all() {
        hub.disconnect(state, who).await;
    }

    let (user_email, real_ip) = {
        let mut connections = state.get_connections().lock().await;
        let email = connections.get(&who).and_then(|c| c.user_email.clone());
        let ip = connections.get(&who).map(|c| c.real_ip.clone()).unwrap_or_else(|| who.ip().to_string());
        connections.remove(&who);
        (email, ip)
    };
    state.broadcast_to_admins(
        crate::structs::ws::WsEvent::UserLeft,
        serde_json::json!({ "addr": who.to_string(), "real_ip": real_ip, "user_email": user_email }),
    );
}
