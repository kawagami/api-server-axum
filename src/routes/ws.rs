use crate::{
    errors::AppError,
    middleware::auth,
    repositories::redis as redis_repo,
    state::{AppState, DisplayTrackedConnection, TrackedConnection},
    structs::{
        auth::{AuthenticatedUser, Claims},
        roles::Perm,
    },
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
    routing::{any, get},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use futures_util::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc, time::SystemTime};
use tokio::{sync::Mutex, time::Duration};

// --- WebSocket Ping-Pong 設定 ---
const PING_INTERVAL_SECONDS: u64 = 30;

#[derive(serde::Deserialize)]
struct WsQuery {
    jwt: Option<String>,
}

async fn validate_ws_token(state: &AppState, token: String) -> Option<String> {
    let jwt_secret = std::env::var("JWT_SECRET").ok()?;
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .ok()?;
    let email = token_data.claims.sub;
    let key = format!("user:login:{}", email);
    let exists = redis_repo::redis_check_key_exists(state, &key)
        .await
        .unwrap_or(false);
    exists.then_some(email)
}

pub fn new(state: AppState) -> Router<AppState> {
    let admin_routes = Router::new()
        .route("/get_online_connections", get(get_online_connections))
        .route("/say_something_to_someone", get(say_something_to_someone))
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
    let user_email = match query.jwt {
        Some(jwt) => validate_ws_token(&state, jwt).await,
        None => None,
    };
    let real_ip = req_headers
        .get("CF-Connecting-IP")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| addr.ip().to_string());
    tracing::info!("{real_ip} connected ({}) email={:?}", user_agent, user_email);
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

    state.broadcast(
        crate::structs::ws::WsEvent::UserJoined,
        serde_json::json!({ "addr": who.to_string(), "real_ip": real_ip, "user_email": user_email }),
    );

    let mut rx = state.get_tx().subscribe();

    // --- send_task: 將 state 的訊息發送給客戶端 ---
    let send_sender_clone = sender_arc.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let mut sender_guard = send_sender_clone.lock().await;
            if let Err(e) = sender_guard.send(Message::Text(msg.into())).await {
                tracing::warn!("Failed to send message to {}: {}", who, e);
                break;
            }
        }
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

    tracing::info!("Websocket context {who} ({real_ip}) destroyed");
}

fn process_message(msg: Message, who: SocketAddr, state: &AppState) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            let _ = state.get_tx().send(format!("{} : {}", who, t));
        }
        Message::Binary(_) => {}
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
    Query(params): Query<SendMessageParams>,
    State(state): State<AppState>,
) -> Result<Json<String>, AppError> {
    auth_user.require_permission(Perm::WsRead)?;
    let connections = state.get_connections().lock().await;

    match params.addr.parse::<SocketAddr>() {
        Ok(socket_addr) => {
            if let Some(tracked_conn) = connections.get(&socket_addr) {
                let mut sender_guard = tracked_conn.sender.lock().await;
                let message = Message::Text(params.message.into());

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

async fn cleanup_connection(state: &AppState, who: SocketAddr) {
    let (user_email, real_ip) = {
        let mut connections = state.get_connections().lock().await;
        let email = connections.get(&who).and_then(|c| c.user_email.clone());
        let ip = connections.get(&who).map(|c| c.real_ip.clone()).unwrap_or_else(|| who.ip().to_string());
        connections.remove(&who);
        (email, ip)
    };
    state.broadcast(
        crate::structs::ws::WsEvent::UserLeft,
        serde_json::json!({ "addr": who.to_string(), "real_ip": real_ip, "user_email": user_email }),
    );
}
