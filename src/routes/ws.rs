use crate::state::AppStateV2;
use axum::extract::connect_info::ConnectInfo;
use axum::routing::any;
use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    Router,
};
use axum_extra::{headers, TypedHeader};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", any(ws_handler))
}

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
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
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: AppStateV2) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        tracing::info!("Pinged {who}...");
    } else {
        tracing::info!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    // By splitting socket we can send and receive at the same time. In this example we will send
    // unsolicited messages to client based on some sort of server's internal event (i.e .timer).
    let (mut sender, mut receiver) = socket.split();

    //
    let mut rx = state.get_tx().subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // 用於追踪最後一次收到 ping/pong 的時間
    let last_ping_time = Arc::new(Mutex::new(Instant::now()));
    let last_ping_time_clone = last_ping_time.clone();

    // This second task will receive messages from client and print them on server console
    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;

            // 如果收到 ping 或 pong，更新時間戳
            match &msg {
                Message::Ping(_) | Message::Pong(_) => {
                    *last_ping_time_clone.lock().await = Instant::now();
                    tracing::debug!("Updated last ping time for {who}");
                }
                _ => {}
            }

            // print message and break if instructed to do so
            if process_message(msg, who, &state).is_break() {
                break;
            }
        }
        cnt
    });

    // 添加超時檢查任務
    let mut timeout_task = tokio::spawn(async move {
        let timeout_duration = Duration::from_secs(60); // 60 秒超時

        loop {
            tokio::time::sleep(Duration::from_secs(10)).await; // 每10秒檢查一次

            let last_ping = *last_ping_time.lock().await;
            let elapsed = last_ping.elapsed();

            if elapsed > timeout_duration {
                tracing::warn!(
                    "Client {who} exceeded ping timeout ({elapsed:?}), closing connection"
                );
                return true; // 超時
            }
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(_) => tracing::info!("send_task end"),
                Err(a) => tracing::info!("Error sending messages {a:?}")
            }
            recv_task.abort();
            timeout_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => tracing::info!("Received {b} messages"),
                Err(b) => tracing::info!("Error receiving messages {b:?}")
            }
            send_task.abort();
            timeout_task.abort();
        },
        rv_c = (&mut timeout_task) => {
            match rv_c {
                Ok(true) => tracing::info!("Connection {who} timed out due to no ping"),
                Ok(false) => tracing::info!("Timeout task completed normally"),
                Err(e) => tracing::info!("Error in timeout task {e:?}")
            }
            send_task.abort();
            recv_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    tracing::info!("Websocket context {who} destroyed");
}

/// helper to print contents of messages to stdout. Has special treatment for Close.
fn process_message(msg: Message, who: SocketAddr, state: &AppStateV2) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            let _ = state.get_tx().send(t.to_string());
        }
        Message::Binary(d) => {
            tracing::info!(">>> {who} sent {} bytes: {d:?}", d.len());
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

        Message::Pong(v) => {
            tracing::info!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            tracing::info!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}
