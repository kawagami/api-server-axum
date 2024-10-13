use crate::state::AppState;
use crate::structs::ws::WsMessage;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub from: String,
    pub to: To,
}

impl ChatMessage {
    pub fn new_jsonstring(content: String, from: String, to: To) -> String {
        let send_json = ChatMessage { content, from, to };
        serde_json::to_string(&send_json).expect("產生 json string 失敗")
    }
    pub fn decode(raw_json_string: &str) -> ChatMessage {
        serde_json::from_str(&raw_json_string).expect("decode raw json string 失敗")
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum To {
    All,
    Private(String), // 這裡的 String 表示特定使用者的 token 或 username
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<QueryParams>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    if is_valid_token(&params.token, &state).await {
        ws.on_upgrade(|socket| websocket(socket, state, params.token))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Invalid token").into_response()
    }
}

async fn is_valid_token(token: &str, state: &Arc<Mutex<AppState>>) -> bool {
    // 在這裡檢查 token 的規則
    // 例如，檢查 token 是否為特定格式或值
    let user_set = &mut state.lock().await.user_set;

    if user_set.contains(token) {
        tracing::debug!("{}", "token 已經存在");
        return false;
    }
    user_set.insert(token.to_string());

    tracing::debug!("{:?}", user_set);
    true
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(stream: WebSocket, state: Arc<Mutex<AppState>>, token: String) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.lock().await.tx.subscribe();

    // Now send the "joined" message to all subscribers.
    let msg = format!("{token} joined.");
    tracing::debug!("{msg}");

    // 加入 ws 時發出的訊息
    let join_msg = ChatMessage::new_jsonstring(msg, token.clone(), To::All);
    let _ = state.lock().await.tx.send(join_msg);

    // clone 要 move 至不同部分的資料
    let token_clone = token.clone();

    // server 端的 send task
    // 運用 subscribe 之後的 rx
    // 將要傳遞的訊息使用 split 出來的 sender 發送給這個訂閱者
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let data_msg = ChatMessage::decode(&msg);

            match &data_msg.to {
                To::All => {
                    // 發送給所有用戶
                    let send_msg = ChatMessage::new_jsonstring(
                        data_msg.content.clone(),
                        data_msg.from.clone(),
                        To::All,
                    );

                    if sender.send(Message::Text(send_msg)).await.is_err() {
                        break;
                    }
                }
                To::Private(target_user) => {
                    // 發送給特定用戶
                    let send_msg = ChatMessage::new_jsonstring(
                        data_msg.content.clone(),
                        data_msg.from.clone(),
                        To::Private(target_user.clone()),
                    );

                    // 如果目標用戶是自己，也發送訊息
                    if target_user == &token_clone || data_msg.from == token_clone {
                        if sender.send(Message::Text(send_msg)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let cp_state = state.clone();

    // server 端的 recv task
    // 接收 client sent 的資料
    // 在這邊控制要怎麼處理
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // 解析接收到的消息
            let data_msg = ChatMessage::decode(&text);

            // 將訊息存入歷史記錄
            let ws_message = WsMessage {
                message: data_msg.content.clone(),
                from: data_msg.from.clone(),
            };
            cp_state
                .lock()
                .await
                .fixed_message_container
                .add(ws_message);

            let send_msg = ChatMessage::new_jsonstring(
                data_msg.content.clone(),
                data_msg.from.clone(),
                data_msg.to.clone(),
            );

            let _ = cp_state.lock().await.tx.send(send_msg);
        }
    });

    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();

            remove_user_set(state.clone(), &token).await;
        },
        _ = &mut recv_task => {
            send_task.abort();

            remove_user_set(state.clone(), &token).await;
        },
    };

    // 組合離開的訊息
    let msg = format!("{token} left.");

    let send_exit_msg = ChatMessage::new_jsonstring(msg, token, To::All);
    let _ = state.lock().await.tx.send(send_exit_msg);
}

async fn remove_user_set(state: Arc<Mutex<AppState>>, token: &str) {
    // 獲取對 AppState 的鎖
    let mut app_state = state.lock().await;
    let user_set = &mut app_state.user_set;

    // 移除 token
    user_set.remove(token);

    let msg = format!("hashset remove {token}.");
    tracing::debug!("{msg}");
}

pub async fn ws_message(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<WsMessage>> {
    // 使用 let 綁定鎖定的值，使其在這個範疇內保持有效
    let state_guard = state.lock().await;

    // 使用已解鎖的固定訊息容器
    let data = state_guard.fixed_message_container.get_all();

    // 將借用的 Vec<&WsMessage> 轉換為 Vec<WsMessage>
    let owned: Vec<WsMessage> = data.into_iter().cloned().collect();

    Json(owned)
}
