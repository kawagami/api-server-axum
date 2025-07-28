use crate::{
    errors::AppError,
    state::{AppStateV2, DisplayTrackedConnection, TrackedConnection},
};
use axum::{
    body::Bytes,
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc, time::SystemTime};
use tokio::{sync::Mutex, time::Duration};

// --- WebSocket Ping-Pong 設定 ---
const PING_INTERVAL_SECONDS: u64 = 30; // 伺服器每 30 秒發送一個 Ping

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", any(ws_handler))
        .route("/get_online_connections", get(get_online_connections))
        .route("/say_something_to_someone", get(say_something_to_someone))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppStateV2>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, state: AppStateV2) {
    let (sender, receiver) = socket.split();
    let sender_arc = Arc::new(Mutex::new(sender));

    let connection_info = TrackedConnection {
        addr: who.to_string(),
        connected_at: SystemTime::now(),
        sender: sender_arc.clone(),
    };

    // 添加連接到狀態中
    {
        let mut connections = state.get_connections().lock().await;
        tracing::info!("connections add new user {}", connection_info.addr);
        connections.insert(who, connection_info);
    }

    let mut rx = state.get_tx().subscribe();

    // --- send_task: 將 state 的訊息發送給客戶端 ---
    let send_sender_clone = sender_arc.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut sender_guard = send_sender_clone.lock().await;
            if let Err(e) = sender_guard.send(Message::Text(msg.into())).await {
                tracing::warn!("Failed to send message to {}: {}", who, e);
                break; // 發送失敗時直接退出，讓 handle_socket 統一清理
            }
        }
        tracing::info!("send_task for {who} ended.");
    });

    // --- recv_task: 接收客戶端訊息 ---
    let recv_state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        let mut receiver = receiver;
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    cnt += 1;
                    if process_message(msg, who, &recv_state_clone).is_break() {
                        break;
                    }
                }
                Err(e) => {
                    tracing::warn!("Error receiving message from {}: {}", who, e);
                    break;
                }
            }
        }
        tracing::info!("recv_task for {who} processed {cnt} messages, ending.");
        cnt
    });

    // --- ping_task: 後端主動發送 Ping 並檢查 Pong 回應 ---
    let ping_sender_clone = sender_arc.clone();
    let mut ping_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECONDS));
        interval.tick().await; // 跳過第一次立即觸發

        loop {
            interval.tick().await;

            tracing::debug!("Sending ping to {who}...");
            {
                let mut sender_guard = ping_sender_clone.lock().await;
                if let Err(e) = sender_guard.send(Message::Ping(Bytes::new())).await {
                    tracing::warn!("Failed to send ping to {who}: {}", e);
                    break; // 發送失敗時直接退出，讓 handle_socket 統一清理
                }
            } // 立即釋放鎖
        }
        tracing::info!("ping_task for {who} ended.");
    });

    // --- tokio::select!: 協調所有任務 ---
    tokio::select! {
        rv_a = (&mut send_task) => {
            if let Err(e) = rv_a {
                tracing::error!("Error in send_task for {who}: {:?}", e);
            }
        },
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
    send_task.abort();
    recv_task.abort();
    ping_task.abort();

    // 最終清理連接
    cleanup_connection(&state, who).await;

    tracing::info!("Websocket context {who} destroyed");
}

fn process_message(msg: Message, who: SocketAddr, state: &AppStateV2) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            let _ = state.get_tx().send(format!("{} : {}", who, t));
        }
        Message::Binary(d) => {
            tracing::debug!(">>> {who} sent {} bytes: {d:?}", d.len());
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                tracing::info!(
                    ">>> {who} sent close with code {} and reason `{}`",
                    cf.code,
                    cf.reason
                );
            } else {
                tracing::info!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        Message::Pong(_) => {
            tracing::debug!("Received pong from {who}");
            // 可以在這裡更新最後 pong 時間
        }
        Message::Ping(_) => {
            tracing::debug!("Received ping from {who}");
        }
    }
    ControlFlow::Continue(())
}

// 原有的獲取所有連接的端點
async fn get_online_connections(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<DisplayTrackedConnection>>, AppError> {
    let connections = state.get_connections().lock().await;

    let result = connections
        .iter()
        .map(|(addr, info)| DisplayTrackedConnection {
            addr: addr.to_string(),
            connected_at: info.connected_at,
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
    Query(params): Query<SendMessageParams>,
    State(state): State<AppStateV2>,
) -> Result<Json<String>, AppError> {
    let connections = state.get_connections().lock().await;

    match params.addr.parse::<SocketAddr>() {
        Ok(socket_addr) => {
            if let Some(tracked_conn) = connections.get(&socket_addr) {
                let mut sender_guard = tracked_conn.sender.lock().await;
                let message = Message::Text(params.message.into());

                match sender_guard.send(message).await {
                    Ok(_) => {
                        tracing::info!("Successfully sent message to {}", socket_addr);
                        Ok(Json("Message sent successfully".to_string()))
                    }
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

// 改進的清理函數
async fn cleanup_connection(state: &AppStateV2, who: SocketAddr) {
    let mut connections = state.get_connections().lock().await;
    if let Some(_removed_conn) = connections.remove(&who) {
        tracing::info!(
            "Removed connection {} from connections map (remaining: {})",
            who,
            connections.len()
        );
    } else {
        tracing::debug!(
            "Connection {} was already removed from connections map",
            who
        );
    }
}
