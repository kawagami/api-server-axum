use crate::games::snake::model::{ClientMessage, ServerMessage, SnakeGameState};
use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_socket(socket: WebSocket) {
    let game_state = Arc::new(Mutex::new(SnakeGameState::new()));
    let tick_duration = std::time::Duration::from_millis(100);
    let (mut sender, mut receiver) = socket.split();
    let input_state = Arc::clone(&game_state);

    // 用於控制 tick loop 的標誌
    let is_connection_alive = Arc::new(Mutex::new(true));
    let is_connection_alive_clone = Arc::clone(&is_connection_alive);

    let input_handle = tokio::spawn(async move {
        while let Some(msg_result) = receiver.next().await {
            match msg_result {
                Ok(msg) => {
                    match msg {
                        Message::Text(text) => match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                let mut state = input_state.lock().await;
                                state.handle_input(client_msg);
                            }
                            Err(e) => {
                                tracing::error!("JSON 解析失敗: {}", e);
                                tracing::error!("原始文字: {}", text);
                            }
                        },
                        Message::Close(_) => {
                            tracing::info!("接收到關閉連接請求");
                            *is_connection_alive_clone.lock().await = false;
                            break; // 收到關閉請求時退出循環
                        }
                        Message::Binary(bin) => {
                            tracing::warn!("接收到 Binary message: {:?}", bin);
                        }
                        Message::Ping(_) | Message::Pong(_) => {
                            tracing::info!("接收到心跳訊息");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("WebSocket 接收錯誤: {}", e);
                    *is_connection_alive_clone.lock().await = false;
                    break;
                }
            }
        }

        // 流結束，設置連接狀態為關閉
        *is_connection_alive_clone.lock().await = false;
        tracing::info!("WebSocket 輸入流結束");
    });

    // 遊戲 tick loop
    while *is_connection_alive.lock().await {
        tokio::time::sleep(tick_duration).await;

        let mut state = game_state.lock().await;

        // 即使遊戲結束，也發送當前狀態（為了讓客戶端知道遊戲結束）
        if !state.game_over {
            state.update();
        }

        let server_msg = ServerMessage::from(&*state);
        let json = serde_json::to_string(&server_msg).unwrap();

        // 發送失敗時設置連接狀態為關閉
        if sender.send(Message::Text(json.into())).await.is_err() {
            tracing::warn!("WebSocket 發送失敗，關閉連接");
            *is_connection_alive.lock().await = false;
            break;
        }

        // 釋放鎖
        drop(state);
    }

    // 等待輸入處理完成
    if let Err(e) = input_handle.await {
        tracing::error!("輸入處理任務失敗: {}", e);
    }

    tracing::info!("WebSocket 連接已關閉");
}
