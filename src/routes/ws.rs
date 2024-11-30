use crate::{
    state::AppStateV2,
    structs::{
        chat::{GetParams, QueryParams},
        ws::{ChatMessage, ChatMessageType, DbChatMessage, To},
    },
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::FixedOffset;
use futures::{sink::SinkExt, stream::StreamExt}; // 提供非同步流處理功能

// 處理 WebSocket 升級請求的 handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade, // 表示 WebSocket 升級請求
    Query(params): Query<QueryParams>,
    State(state): State<AppStateV2>,
) -> impl IntoResponse {
    // 檢查 token 是否有效，如果有效則進行 WebSocket 升級
    if is_valid_token(&params.token, &state).await {
        ws.on_upgrade(|socket| websocket(socket, state, params.token)) // 啟動 WebSocket 連線
    } else {
        // 如果 token 無效，返回錯誤回應
        (StatusCode::INTERNAL_SERVER_ERROR, "Invalid token").into_response()
    }
}

// 驗證 token 是否有效
async fn is_valid_token(token: &str, state: &AppStateV2) -> bool {
    if state
        .check_member_exists("online_members", token) // 檢查 token 是否已存在於線上成員
        .await
        .unwrap()
    {
        tracing::debug!("{}", "token 已經存在");
        return false; // 如果已存在，返回 false
    }
    let _ = state.redis_zadd("online_members", token).await; // 將 token 加入線上成員集合
    true // 返回 true 表示 token 有效
}

// 處理單一 WebSocket 連線的邏輯
// 即單一連接的客戶端/用戶，我們將為其產生兩個獨立的任務（對於接收/傳送聊天訊息）。
async fn websocket(stream: WebSocket, state: AppStateV2, token: String) {
    // 分割 WebSocket 流，便於同時處理發送與接收
    let (mut sender, mut receiver) = stream.split();

    // 訂閱訊息，確保之後的加入訊息也能顯示給客戶端
    let mut rx = state.get_tx().subscribe();

    // 當用戶加入時，通知所有在線成員
    let join_users_set = state
        .redis_zrange("online_members") // 獲取所有線上成員
        .await
        .unwrap()
        .0
        .join(",");
    let join_msg = ChatMessage::new_jsonstring(
        ChatMessageType::Join, // 訊息類型為加入
        join_users_set,
        token.clone(),
        To::All, // 傳送給所有人
    );
    let _ = state.get_tx().send(join_msg); // 廣播加入訊息

    // clone token，傳遞給需要移動的任務
    let token_clone = token.clone();

    // 伺服器端發送訊息的任務
    // 運用 subscribe 之後的 rx
    // 將要傳遞的訊息使用 split 出來的 sender 發送給這個訂閱者
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // 從訊息通道接收訊息
            let data_msg = ChatMessage::decode(&msg);

            match &data_msg.to {
                To::All => {
                    // 發送給所有用戶
                    let send_msg = ChatMessage::new_jsonstring(
                        data_msg.message_type,
                        data_msg.content,
                        data_msg.from,
                        To::All,
                    );

                    if sender.send(Message::Text(send_msg)).await.is_err() {
                        break; // 如果發送失敗，結束任務
                    }
                }
                To::Private(target_user) => {
                    // 發送給特定用戶
                    let send_msg = ChatMessage::new_jsonstring(
                        data_msg.message_type,
                        data_msg.content,
                        data_msg.from.clone(),
                        To::Private(target_user.clone()),
                    );

                    // 如果目標是自己，也需要接收訊息
                    if target_user == &token_clone || data_msg.from == token_clone {
                        if sender.send(Message::Text(send_msg)).await.is_err() {
                            break;
                        }
                    }
                }
                To::Myself => {} // 忽略自己
            }
        }
    });

    // 接收客戶端訊息的任務
    let cp_state = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let data_msg = ChatMessage::decode(&text);

            // 將全體訊息存入資料庫
            if data_msg.to == To::All {
                let _ = &cp_state
                    .insert_chat_message("Message", "All", &data_msg.from, &data_msg.content)
                    .await;
            }

            // 廣播接收到的訊息
            let send_msg = ChatMessage::new_jsonstring(
                data_msg.message_type,
                data_msg.content,
                data_msg.from,
                data_msg.to,
            );
            let _ = cp_state.get_tx().send(send_msg);
        }
    });

    // 如果任務之一完成或中止，另一個也會被中止
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
            remove_user_set(&state, &token).await; // 移除用戶
        },
        _ = &mut recv_task => {
            send_task.abort();
            remove_user_set(&state, &token).await; // 移除用戶
        },
    };

    // 用戶離開時廣播訊息
    let leave_users_set = state
        .redis_zrange("online_members")
        .await
        .unwrap()
        .0
        .join(",");
    let send_exit_msg =
        ChatMessage::new_jsonstring(ChatMessageType::Leave, leave_users_set, token, To::All);
    let _ = state.get_tx().send(send_exit_msg);
}

// 移除用戶資料
async fn remove_user_set(state: &AppStateV2, token: &str) {
    let _ = state.redis_zrem("online_members", token).await; // 從 Redis 刪除用戶 token
}

// 處理查詢聊天訊息的 handler
pub async fn ws_message(
    State(state): State<AppStateV2>,
    Query(params): Query<GetParams>,
) -> Json<Vec<ChatMessage>> {
    // 設定查詢訊息的上限，預設為 10
    let limit = params.limit.unwrap_or(10);

    let messages: Vec<DbChatMessage> = sqlx::query_as(
        r#"
            SELECT
                id,
                message_type,
                to_type,
                user_name,
                message,
                created_at
            FROM
                chat_messages
            ORDER BY
                id DESC
            LIMIT
                $1
        "#,
    )
    .bind(limit)
    .fetch_all(&state.get_pool())
    .await
    .unwrap();

    // 將資料庫訊息轉換為客戶端格式的訊息
    let chat_messages: Vec<ChatMessage> = messages
        .into_iter()
        .map(|db_msg| {
            let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
            let created_at_plus_8 = db_msg.created_at.with_timezone(&utc_plus_8);
            let created_at_str = created_at_plus_8.format("%Y-%m-%d %H:%M:%S").to_string();

            ChatMessage {
                message_type: ChatMessageType::Message,
                content: db_msg.message,
                from: db_msg.user_name,
                to: To::All,
                created_at: created_at_str,
            }
        })
        .collect();

    Json(chat_messages)
}
