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
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
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
    let _ = state.lock().await.tx.send(msg);

    // Spawn the first task that will receive broadcast messages and send text
    // messages over the websocket to our client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Clone things we want to pass (move) to the receiving task.
    let tx = state.lock().await.tx.clone();
    let name = token.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{name}: {text}"));
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
