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

#[derive(Serialize)]
pub struct SendJson {
    pub content: String,
    pub from: String,
    pub to: To,
}

#[derive(Deserialize)]
pub struct ReceiveJson {
    pub content: String,
    pub from: String,
    pub to: To,
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
    let raw_join_msg = SendJson {
        content: msg,
        from: token.clone(),
        to: To::All,
    };
    let join_msg = serde_json::to_string(&raw_join_msg).expect("send_task 解析 send_msg 失敗");
    let _ = state.lock().await.tx.send(join_msg);

    // clone 要 move 至不同部分的資料
    let token_clone = token.clone();

    // server 端的 send task
    // 運用 subscribe 之後的 rx
    // 將要傳遞的訊息使用 split 出來的 sender 發送給這個訂閱者
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let data_msg: ReceiveJson =
                serde_json::from_str(&msg).expect("send_task 解析 data_msg 失敗");

            let to_send_msg = |to: To| {
                let raw_send_msg = SendJson {
                    content: data_msg.content.clone(),
                    from: data_msg.from.clone(),
                    to,
                };
                serde_json::to_string(&raw_send_msg).expect("send_task 解析 send_msg 失敗")
            };

            match &data_msg.to {
                To::All => {
                    // 發送給所有用戶
                    let send_msg = to_send_msg(To::All);
                    if sender.send(Message::Text(send_msg)).await.is_err() {
                        break;
                    }
                }
                To::Private(target_user) => {
                    // 發送給特定用戶
                    let send_msg = to_send_msg(To::Private(target_user.clone()));

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
            let data_msg: ReceiveJson =
                serde_json::from_str(&text).expect("recv_task 解析 data_msg 失敗");

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

            // 組合要發送的訊息
            let raw_send_msg = SendJson {
                content: data_msg.content.clone(),
                from: data_msg.from.clone(),
                to: data_msg.to.clone(),
            };

            let send_msg =
                serde_json::to_string(&raw_send_msg).expect("recv_task 解析 send_msg 失敗");

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
    let raw_send_msg = SendJson {
        content: msg.clone(),
        from: token,
        to: To::All,
    };

    let send_exit_msg = serde_json::to_string(&raw_send_msg).expect("組合 send_exit_msg 失敗");
    tracing::debug!("{msg}");
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
