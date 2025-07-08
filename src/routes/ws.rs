use crate::{
    errors::AppError,
    state::{AppStateV2, DisplayTrackedConnection, TrackedConnection},
};
use axum::{
    body::Bytes,
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc};
use tokio::{sync::Mutex, time::Duration};

// --- WebSocket Ping-Pong 設定 ---
const PING_INTERVAL_SECONDS: u64 = 30; // 伺服器每 30 秒發送一個 Ping

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", any(ws_handler))
        .route("/", get(get_online_connections))
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
    // 使用 Arc<Mutex> 包裝 sender，以便在多個任務間共享
    let sender_arc = Arc::new(Mutex::new(sender));

    let connection_info = TrackedConnection {
        addr: who.to_string(),
        connected_at: std::time::SystemTime::now(),
        sender: sender_arc.clone(), // 存下控制 sender
    };

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
            // 鎖定 sender 並發送訊息
            let mut sender_guard = send_sender_clone.lock().await;
            if sender_guard.send(Message::text(msg)).await.is_err() {
                break; // 如果發送失敗，表示連線可能已關閉
            }
        }
        tracing::info!("send_task for {who} ended.");
    });

    // --- recv_task: 接收客戶端訊息 ---
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        let mut receiver = receiver; // 獲取 receiver 的所有權
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;

            // 處理客戶端傳來的訊息
            if process_message(msg, who, &state).is_break() {
                break;
            }
        }
        tracing::info!("recv_task for {who} processed {cnt} messages, ending.");
        cnt
    });

    // --- ping_task: 後端主動發送 Ping 並檢查 Pong 回應 ---
    let ping_sender_clone = sender_arc.clone();
    let mut ping_task = tokio::spawn(async move {
        // 設定定時器，第一次立即執行，避免等待第一個間隔
        let mut interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL_SECONDS));
        interval.tick().await;

        loop {
            interval.tick().await; // 等待下一個 ping 間隔

            tracing::debug!("Sending ping to {who}...");
            let mut sender_guard = ping_sender_clone.lock().await;
            if sender_guard
                .send(Message::Ping(Bytes::new()))
                .await
                .is_err()
            {
                tracing::warn!("Failed to send ping to {who}, connection might be closed.");
                break; // 如果無法發送 ping，表示連線可能已關閉，終止任務
            }
            drop(sender_guard); // 盡快釋放鎖
        }
        tracing::info!("ping_task for {who} ended.");
    });

    // --- tokio::select!: 協調所有任務 ---
    // 如果任何一個任務結束，就終止其他的任務。
    tokio::select! {
        // send_task 結束
        rv_a = (&mut send_task) => {
            if let Err(e) = rv_a {
                tracing::error!("Error in send_task for {who}: {:?}", e);
            }
        },
        // recv_task 結束
        rv_b = (&mut recv_task) => {
            if let Err(e) = rv_b {
                tracing::error!("Error in recv_task for {who}: {:?}", e);
            }
        },
        // ping_task 結束 (通常是因為超時或發送失敗)
        rv_c = (&mut ping_task) => {
            if let Err(e) = rv_c {
                tracing::error!("Error in ping_task for {who}: {:?}", e);
            }
        }
    }

    // 任何一個任務結束，都會導致 tokio::select! 塊完成，
    // 然後我們在這裡手動中止所有 remaining tasks，確保資源被釋放。
    send_task.abort();
    recv_task.abort();
    ping_task.abort();

    // returning from the handler closes the websocket connection
    tracing::info!("Websocket context {who} destroyed");
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
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
        Message::Pong(_) => {}
        Message::Ping(_) => {}
    }
    ControlFlow::Continue(())
}

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
