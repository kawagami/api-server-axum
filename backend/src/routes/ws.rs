use crate::extract::{Json, Query};
use crate::{
    errors::{AppError, RequestError, SystemError},
    middleware::auth,
    repositories::redis as redis_repo,
    state::{AppState, DisplayTrackedConnection, TrackedConnection},
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    body::Bytes,
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, State
    },
    http::HeaderMap,
    middleware,
    response::IntoResponse,
    routing::{any, get, post},
    Router
};
use axum_extra::{headers, TypedHeader};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc, time::SystemTime};
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

#[derive(serde::Deserialize)]
struct WsQuery {
    ticket: Option<String>,
}

/// 連線時間對外一律用固定寬度的 ISO-8601 毫秒 UTC 字串
fn to_iso(t: SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

pub fn new(state: AppState) -> Router<AppState> {
    // 刻意直接掛 authorize_and_load 而非 super::with_auth（＝不進 audit）：/ticket 是
    // 每次 WS 重連都會打的高頻端點，寫進 admin_audit_logs 只會把稽核表灌滿噪音。
    // 其餘 admin 模組一律用 with_auth。
    let admin_routes = Router::new()
        .route("/connections", get(list_connections))
        .route("/messages", post(send_message))
        .route("/ticket", post(create_ws_ticket))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ));

    Router::new()
        .route("/", any(ws_handler))
        .merge(admin_routes)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(query): Query<WsQuery>,
    req_headers: HeaderMap,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    // admin 身分改用一次性 ticket（POST /ws/ticket 換發，30 秒 TTL），
    // JWT 不再走 URL query，避免 token 進 access log
    let user_email = match query.ticket {
        Some(ticket) => redis_repo::consume_ws_ticket(state.get_redis_pool(), &ticket)
            .await
            .ok()
            .flatten(),
        None => None,
    };
    // 與 middleware/rate_limit.rs 同一條規則（同一個函式）：只有確定流量都經 Cloudflare
    // （TRUST_CF_HEADER=true）才信任這個 header
    let real_ip = crate::utils::net::client_ip(
        state.get_config().trust_cf_header,
        &req_headers,
        Some(addr.ip()),
    );
    // debug：每個訪客一行、且帶 IP / UA / email 個資。連線清單走 GET /ws/connections，
    // 新連線走 user_joined（只推 admin），這行只是本機開發時的方便，不該進生產 stdout。
    tracing::debug!("{real_ip} connected ({}) email={:?}", user_agent, user_email);

    // 每日不重複到訪統計：以 WS 握手為採集點（天然濾掉不跑 JS 的 bot），
    // 去重元素 = ip|ua。best-effort，不阻塞連線。
    {
        let redis_pool = state.get_redis_pool().clone();
        let ip = real_ip.clone();
        let ua = user_agent.clone();
        tokio::spawn(async move {
            crate::repositories::visitors::record_visit(&redis_pool, &ip, &ua).await;
        });
    }

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, user_email, real_ip, user_agent))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, state: AppState, user_email: Option<String>, real_ip: String, user_agent: String) {
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

    // --- recv_task: 接收客戶端訊息 ---
    let recv_state_clone = state.clone();
    let recv_last_pong = last_pong.clone();
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        let mut receiver = receiver;
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    cnt += 1;
                    // 存活證明只認 Pong（其餘訊息可能來自沒在讀我們 ping 的客戶端）
                    if matches!(msg, Message::Pong(_)) {
                        *recv_last_pong.lock().unwrap() = Instant::now();
                    }
                    if process_message(msg, who, &recv_state_clone).await.is_break() {
                        break;
                    }
                }
                Err(e) => {
                    // debug 不是 warn：這裡幾乎清一色是 "Connection reset without closing
                    // handshake" —— 關分頁、手機睡眠、NAT 逾時都會產生，是公開網站的常態
                    // （實測佔了 logs 表 WARN+ 的 45%）。而且斷線後下面照樣走完整清理，
                    // 沒有任何要人介入的事。真正需要注意的送出失敗另有其他 log。
                    tracing::debug!("Error receiving message from {}: {}", who, e);
                    break;
                }
            }
        }
        cnt
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
                break;
            }

            {
                let mut sender_guard = ping_sender_clone.lock().await;
                if let Err(e) = sender_guard.send(Message::Ping(Bytes::new())).await {
                    tracing::warn!("Failed to send ping to {who}: {}", e);
                    break;
                }
            }
        }
    });

    // --- tokio::select!: 協調所有任務 ---
    tokio::select! {
        rv_b = (&mut recv_task) => {
            if let Err(e) = rv_b {
                tracing::error!("Error in recv_task for {who}: {:?}", e);
            }
        },
        rv_c = (&mut ping_task) => {
            if let Err(e) = rv_c {
                tracing::error!("Error in ping_task for {who}: {:?}", e);
            }
        }
    }

    // 清理工作
    recv_task.abort();
    ping_task.abort();

    // 最終清理連接
    cleanup_connection(&state, who).await;

    tracing::debug!("Websocket context {who} ({real_ip}) destroyed");
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

// 原有的獲取所有連接的端點
async fn list_connections(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DisplayTrackedConnection>>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;

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

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct SendMessageParams {
    pub addr: String,
    pub message: String,
}

/// 失敗一律回非 2xx。舊版對「位址格式錯 / 連線不存在 / 送出失敗」都回 200 加一段錯誤字串，
/// 呼叫端只看 status 的話會把失敗顯示成成功。
async fn send_message(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(params): Json<SendMessageParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;

    // 這兩個分支不另外記 log：錯誤本身已經回給呼叫端，也已經由 errors.rs 統一記下
    // （帶 request_id），再印一行 info 只是同一件事的第二份，而且 info 不落地 logs 表。
    let socket_addr = params.addr.parse::<SocketAddr>().map_err(|_| {
        RequestError::InvalidContent(format!("無效的連線位址格式：{}", params.addr))
    })?;

    let connections = state.get_connections().lock().await;
    let tracked_conn = connections
        .get(&socket_addr)
        .ok_or(RequestError::NotFound)?;

    // 事件名一律走 WsEvent enum，不要在這裡手寫字串 ——
    // 手寫的那份不會出現在 enum 裡，前端對照表也就跟著漏掉（admin_message 原本就是這樣走丟的）
    let payload = crate::structs::ws::envelope(
        crate::structs::ws::WsEvent::AdminMessage.as_str(),
        serde_json::json!({ "content": params.message, "from": auth_user.name }),
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

    Ok(Json(serde_json::json!({ "sent": true })))
}

/// 換發 WS 一次性連線票（30 秒 TTL）。登入中的 admin 用它連 WS 取得管理員身分，
/// token 本體不再出現在 WS URL。
///
/// 權限門檻必須與 `list_connections` 一致（同為 `ws:read`）：ticket 換來的連線會被
/// 標成 admin 身分，因而收得到 `broadcast_to_admins` 推的 `user_joined` / `user_left`，
/// 那兩個事件的 payload 含 `real_ip` 與 `user_email`。少了這道檢查，沒有 ws:read 的管理員
/// HTTP 端查不到連線清單，卻能改走 WS 拿到同樣的個資。
async fn create_ws_ticket(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;

    let ticket = uuid::Uuid::new_v4().to_string();
    redis_repo::set_ws_ticket(state.get_redis_pool(), &ticket, &auth_user.name).await?;
    Ok(Json(serde_json::json!({ "ticket": ticket })))
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
