use crate::{
    errors::{AppError, SystemError, WebSocketError},
    repositories::{redis, ws},
    state::AppStateV2,
    structs::ws::{ChatMessage, ChatMessageType, GetParams, QueryParams, To},
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::FixedOffset;
use futures::{sink::SinkExt, stream::StreamExt};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", get(websocket_handler))
        .route("/messages", get(ws_message))
}

// WebSocket 處理函數
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<QueryParams>,
    State(state): State<AppStateV2>,
) -> Result<impl IntoResponse, AppError> {
    // 驗證 token
    validate_token(&params.token, &state).await?;
    Ok(ws.on_upgrade(move |socket| websocket(socket, state, params.token)))
}

// 驗證 token
async fn validate_token(token: &str, state: &AppStateV2) -> Result<(), WebSocketError> {
    let is_member_exists = redis::check_member_exists(state, "online_members", token)
        .await
        .map_err(|e| WebSocketError::UserManagementFailed(e.to_string()))?;

    if is_member_exists {
        tracing::debug!("token已經存在");
        return Err(WebSocketError::InvalidToken);
    }

    // 新增用戶至線上列表
    redis::redis_zadd(state, "online_members", token)
        .await
        .map_err(|e| WebSocketError::ConnectionFailed(e.to_string()))?;

    Ok(())
}

// 處理 WebSocket 連線
async fn websocket(stream: WebSocket, state: AppStateV2, token: String) {
    let (mut sender, mut receiver) = stream.split();
    let mut rx = state.get_tx().subscribe();

    // 廣播用戶加入訊息
    if let Err(e) = broadcast_join_message(&state, &token).await {
        tracing::error!("廣播加入訊息失敗: {}", e);
        return;
    }

    // 創建發送與接收任務
    let token_clone = token.clone();
    let mut send_task =
        tokio::spawn(
            async move { process_outgoing_messages(&mut rx, &mut sender, token_clone).await },
        );
    let cp_state = state.clone();
    let mut recv_task =
        tokio::spawn(async move { process_incoming_messages(&mut receiver, &cp_state).await });

    // 等待任一任務結束後中止另一個
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        },
        _ = &mut recv_task => {
            send_task.abort();
        },
    };

    // 清理用戶資源
    if let Err(e) = remove_user(&state, &token).await {
        tracing::error!("移除用戶失敗: {}", e);
    }

    // 廣播用戶離開訊息
    let _ = broadcast_leave_message(&state, &token).await;
}

// 廣播用戶加入訊息
async fn broadcast_join_message(state: &AppStateV2, token: &str) -> Result<(), AppError> {
    let users_result = redis::redis_zrange(state, "online_members")
        .await
        .map_err(|e| WebSocketError::UserManagementFailed(e.to_string()))?;

    let join_users_set = users_result.0.join(",");

    state
        .get_tx()
        .send(
            ChatMessage::new(
                None,
                ChatMessageType::Join,
                join_users_set,
                token.to_string(),
                To::All,
            )
            .to_json_string()
            .map_err(AppError::from)?,
        )
        .map_err(|e| WebSocketError::BroadcastFailed(e.to_string()))?;

    Ok(())
}

// 處理傳出訊息
async fn process_outgoing_messages(
    rx: &mut tokio::sync::broadcast::Receiver<String>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    token: String,
) {
    while let Ok(msg) = rx.recv().await {
        let data_msg = match ChatMessage::decode(&msg) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("解析訊息失敗: {}", e);
                continue;
            }
        };

        let should_send = match &data_msg.to {
            To::All => true,
            To::Private(target_user) => target_user == &token || data_msg.from == token,
            To::Myself => false,
        };

        if should_send {
            let json_msg = match ChatMessage::new(
                None,
                data_msg.message_type,
                data_msg.content,
                data_msg.from,
                data_msg.to,
            )
            .to_json_string()
            {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!("序列化訊息失敗: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json_msg.into())).await.is_err() {
                tracing::error!("發送 WebSocket 訊息失敗");
                break;
            }
        }
    }
}

// 處理接收消息
async fn process_incoming_messages(
    receiver: &mut futures::stream::SplitStream<WebSocket>,
    state: &AppStateV2,
) {
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        let data_msg = match ChatMessage::decode(&text) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("解析接收訊息失敗: {}", e);
                continue;
            }
        };

        // 儲存公開訊息
        if data_msg.to == To::All {
            if let Err(e) =
                ws::insert_chat_message(state, "Message", "All", &data_msg.from, &data_msg.content)
                    .await
            {
                tracing::error!("儲存聊天訊息失敗: {}", e);
            }
        }

        // 廣播訊息
        let json_msg = match ChatMessage::new(
            None,
            data_msg.message_type,
            data_msg.content,
            data_msg.from,
            data_msg.to,
        )
        .to_json_string()
        {
            Ok(j) => j,
            Err(e) => {
                tracing::error!("序列化回應訊息失敗: {}", e);
                continue;
            }
        };

        if let Err(e) = state.get_tx().send(json_msg) {
            tracing::error!("廣播訊息失敗: {}", e);
        }
    }
}

// 移除用戶
async fn remove_user(state: &AppStateV2, token: &str) -> Result<(), WebSocketError> {
    redis::redis_zrem(state, "online_members", token)
        .await
        .map_err(|e| WebSocketError::UserManagementFailed(e.to_string()))
}

// 廣播用戶離開消息
async fn broadcast_leave_message(state: &AppStateV2, token: &str) -> Result<(), AppError> {
    let users_result = redis::redis_zrange(state, "online_members")
        .await
        .map_err(|e| WebSocketError::UserManagementFailed(e.to_string()))?;

    let leave_users_set = users_result.0.join(",");

    state
        .get_tx()
        .send(
            ChatMessage::new(
                None,
                ChatMessageType::Leave,
                leave_users_set,
                token.to_string(),
                To::All,
            )
            .to_json_string()
            .map_err(AppError::from)?,
        )
        .map_err(|e| WebSocketError::BroadcastFailed(e.to_string()))?;

    Ok(())
}

// 獲取歷史訊息
pub async fn ws_message(
    State(state): State<AppStateV2>,
    Query(params): Query<GetParams>,
) -> Result<Json<Vec<ChatMessage>>, AppError> {
    let messages = ws::ws_message(&state, params.limit, params.before_id)
        .await
        .map_err(|e| AppError::SystemError(SystemError::Internal(e.to_string())))?;

    let chat_messages: Vec<ChatMessage> = messages
        .into_iter()
        .map(|db_msg| {
            let created_at = db_msg
                .created_at
                .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap());
            ChatMessage {
                id: Some(db_msg.id.into()),
                message_type: ChatMessageType::Message,
                content: db_msg.message,
                from: db_msg.user_name,
                to: To::All,
                created_at: created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            }
        })
        .collect();

    Ok(Json(chat_messages))
}
