use crate::games::snake::service::handle_socket;
use axum::{
    extract::{Path, Query, WebSocketUpgrade},
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

/// 選擇性：若你希望支援 query，例如玩家 ID、房間等
#[derive(Debug, Deserialize)]
pub struct SnakeQueryParams {
    pub _player_id: Option<String>,
}

/// WebSocket 連線的 handler：對應 `/games/snake`
pub async fn ws_game_handler(
    ws: WebSocketUpgrade,
    Query(_params): Query<SnakeQueryParams>,
    Path(_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        // 可以把參數傳進去 service，如果你有狀態綁定的話
        handle_socket(socket).await;
    })
}
