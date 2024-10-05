use crate::state::AppState;
use axum::extract::State;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    response::IntoResponse,
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
    pub to: To,
}

#[derive(Serialize, Deserialize)]
pub enum To {
    All,
    Private,
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
    // 在这里检查 token 的规则
    // 例如，检查 token 是否为特定格式或值
    let user_set = &mut state.lock().await.user_set;
    // let tx = &state.lock().await.tx;

    if user_set.contains(token) {
        tracing::debug!("{}", "token 已經存在");
        return false;
    }
    user_set.insert(token.to_string());

    // let msg = format!("{token} joined.");
    // let _ = tx.send(msg);

    tracing::debug!("{}", "is_valid_token testing");
    tracing::debug!("{:?}", user_set);
    // token == "expected_token_value"
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

    let send_from = token.clone();

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // handle msg
            let data_msg: ReceiveJson =
                serde_json::from_str(&msg).expect("send_task 解析 data_msg 失敗");

            let raw_send_msg = SendJson {
                content: data_msg.content,
                from: send_from.to_owned(),
                to: To::All,
            };

            let send_msg =
                serde_json::to_string(&raw_send_msg).expect("send_task 解析 send_msg 失敗");

            // In any websocket error, break loop.
            if sender.send(Message::Text(send_msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state.lock().await.tx.clone();
    let name = token.clone();
    let cp_state = Arc::clone(&state);

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // handle msg
            let data_msg: ReceiveJson =
                serde_json::from_str(&text).expect("recv_task 解析 data_msg 失敗");

            // // 將收到的 content 全頻道返回
            // let raw_send_msg = SendJson {
            //     content: data_msg.content,
            //     from: name.to_owned(),
            //     to: To::All,
            // };

            // 依照 data_msg.content input 的 ID 取資料庫的資料
            let response_content = get_hackmd_note_lists_info(
                cp_state.clone(),
                data_msg.content.parse::<i64>().expect("轉換 i64 fail"),
            )
            .await;

            // 組合要發送的訊息
            let raw_send_msg = SendJson {
                content: response_content,
                from: name.to_owned(),
                to: To::All,
            };

            let send_msg =
                serde_json::to_string(&raw_send_msg).expect("recv_task 解析 send_msg 失敗");

            // Add username before message.
            let _ = tx.send(send_msg);
            // let sql_name = get_blogs_info(Arc::clone(&cp_state)).await;
            // let _ = tx.send(format!("{name}: {sql_name}"));
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

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{token} left.");
    tracing::debug!("{msg}");
    let _ = state.lock().await.tx.send(msg);

    // // Remove username from map so new clients can take it again.
    // state.user_set.lock().await.remove(&username);
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

async fn get_hackmd_note_lists_info(state: Arc<Mutex<AppState>>, id: i64) -> String {
    // 獲取對 AppState 的鎖
    let pool = &state.lock().await.pool;
    let row: (String,) = sqlx::query_as("SELECT title FROM hackmd_note_lists WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();

    row.0
}
