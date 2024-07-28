use std::sync::RwLock;

use axum::extract::State;
use hyper::StatusCode;
use serde::Deserialize;

use axum::{
    extract::{Query, ws::{Message, WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

use crate::state::{AppState, SharedState};

#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<QueryParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    if is_valid_token(&params.token) {
        ws.on_upgrade(|socket| websocket(socket, state))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Invalid token").into_response()
    }
}

fn is_valid_token(token: &str) -> bool {
    // 在这里检查 token 的规则
    // 例如，检查 token 是否为特定格式或值
    // token == "expected_token_value"
    true
}

// This function deals with a single websocket connection, i.e., a single
// connected client / user, for which we will spawn two independent tasks (for
// receiving / sending chat messages).
async fn websocket(stream: WebSocket, state: Arc<RwLock<AppState>>) {
    // By splitting, we can send and receive at the same time.
    let (mut sender, mut receiver) = stream.split();

    // Username gets set in the receive loop, if it's valid.
    let mut username = String::new();
    // Loop until a text message is found.
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            // If username that is sent by client is not taken, fill username string.
            check_username(&state, &mut username, &name);

            // If not empty we want to quit the loop else we want to quit function.
            if !username.is_empty() {
                break;
            } else {
                // Only send our client that username is taken.
                let _ = sender
                    .send(Message::Text(String::from("Username already taken.")))
                    .await;

                return;
            }
        }
    }

    // We subscribe *before* sending the "joined" message, so that we will also
    // display it to our client.
    let mut rx = state.read().unwrap().tx.subscribe();

    // Now send the "joined" message to all subscribers.
    let msg = format!("{username} joined.");
    tracing::debug!("{msg}");
    let _ = state.read().unwrap().tx.send(msg);

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
    let tx = state.read().unwrap().tx.clone();
    let name = username.clone();

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
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    // Send "user left" message (similar to "joined" above).
    let msg = format!("{username} left.");
    tracing::debug!("{msg}");
    let _ = state.read().unwrap().tx.send(msg);

    // // Remove username from map so new clients can take it again.
    // state.user_set.lock().unwrap().remove(&username);
}

fn check_username(state: &Arc<RwLock<AppState>>, string: &mut String, name: &str) {
    let user_set = &mut state.write().unwrap().user_set;

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());

        string.push_str(name);
    }
}
