use crate::{
    errors::AppError,
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
        Extension, Query, State,
    },
    http::HeaderMap,
    middleware,
    response::IntoResponse,
    routing::{any, get, post},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc, time::SystemTime};
use tokio::{sync::Mutex, time::Duration};

// --- WebSocket Ping-Pong 設定 ---
const PING_INTERVAL_SECONDS: u64 = 30;

#[derive(serde::Deserialize)]
struct WsQuery {
    ticket: Option<String>,
}

pub fn new(state: AppState) -> Router<AppState> {
    let admin_routes = Router::new()
        .route("/get_online_connections", get(get_online_connections))
        .route("/say_something_to_someone", post(say_something_to_someone))
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
    let real_ip = req_headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| addr.ip().to_string());
    tracing::info!("{real_ip} connected ({}) email={:?}", user_agent, user_email);

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

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, user_email, real_ip))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, state: AppState, user_email: Option<String>, real_ip: String) {
    let (sender, receiver) = socket.split();
    let sender_arc = Arc::new(Mutex::new(sender));

    let connection_info = TrackedConnection {
        connected_at: SystemTime::now(),
        sender: sender_arc.clone(),
        user_email: user_email.clone(),
        real_ip: real_ip.clone(),
    };

    {
        let mut connections = state.get_connections().lock().await;
        connections.insert(who, connection_info);
    }

    // 含 IP / email 個資，只推給 admin 連線，不對匿名訪客廣播
    state.broadcast_to_admins(
        crate::structs::ws::WsEvent::UserJoined,
        serde_json::json!({ "addr": who.to_string(), "real_ip": real_ip, "user_email": user_email }),
    );

    // --- recv_task: 接收客戶端訊息 ---
    let recv_state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        let mut receiver = receiver;
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    cnt += 1;
                    if process_message(msg, who, &recv_state_clone).await.is_break() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Error receiving message from {}: {}", who, e);
                    break;
                }
            }
        }
        cnt
    });

    // --- ping_task: 後端主動發送 Ping 並檢查 Pong 回應 ---
    let ping_sender_clone = sender_arc.clone();
    let mut ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECONDS));
        interval.tick().await; // 跳過第一次立即觸發

        loop {
            interval.tick().await;

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
        Message::Pong(_) => {}
        Message::Ping(_) => {}
    }
    ControlFlow::Continue(())
}

// 原有的獲取所有連接的端點
async fn get_online_connections(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DisplayTrackedConnection>>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;
    let connections = state.get_connections().lock().await;

    let result = connections
        .iter()
        .map(|(addr, info)| DisplayTrackedConnection {
            addr: addr.to_string(),
            connected_at: info.connected_at,
            user_email: info.user_email.clone(),
            real_ip: info.real_ip.clone(),
        })
        .collect();

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct SendMessageParams {
    pub addr: String,
    pub message: String,
}

async fn say_something_to_someone(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(params): Json<SendMessageParams>,
) -> Result<Json<String>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;
    let connections = state.get_connections().lock().await;

    match params.addr.parse::<SocketAddr>() {
        Ok(socket_addr) => {
            if let Some(tracked_conn) = connections.get(&socket_addr) {
                let mut sender_guard = tracked_conn.sender.lock().await;
                let payload = crate::structs::ws::envelope(
                    "admin_message",
                    serde_json::json!({ "content": params.message, "from": auth_user.name }),
                );
                let message = Message::Text(payload.into());

                match sender_guard.send(message).await {
                    Ok(_) => Ok(Json("Message sent successfully".to_string())),
                    Err(e) => {
                        tracing::error!("Failed to send message to {}: {}", socket_addr, e);
                        // 這裡不立即清理連接，讓 handle_socket 中的任務處理
                        Ok(Json(format!("Failed to send message: {}", e)))
                    }
                }
            } else {
                tracing::info!("No connection found for address: {}", params.addr);
                Ok(Json("Connection not found".to_string()))
            }
        }
        Err(_) => {
            tracing::info!("Invalid socket address format: {}", params.addr);
            Ok(Json("Invalid address format".to_string()))
        }
    }
}

/// 換發 WS 一次性連線票（30 秒 TTL）。登入中的 admin 用它連 WS 取得管理員身分，
/// token 本體不再出現在 WS URL。
async fn create_ws_ticket(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
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
