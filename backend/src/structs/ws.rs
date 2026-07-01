use serde_json::{json, Value};

/// 站台 WS 統一信封序列化。應用層訊息一律 `{ type, data }`；
/// 遊戲訊息額外帶 `game` 欄（邏輯頻道）。所有送出端走此處，杜絕格式長歪。
pub fn envelope(typ: &str, data: Value) -> String {
    json!({ "type": typ, "data": data }).to_string()
}

pub fn game_envelope(game: &str, typ: &str, data: Value) -> String {
    json!({ "game": game, "type": typ, "data": data }).to_string()
}

pub enum WsEvent {
    StockCompleted,
    StockFailed,
    BlogCreated,
    UserJoined,
    UserLeft,
    TorrentProgress,
    TorrentCompleted,
    TorrentFailed,
}

impl WsEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StockCompleted => "stock_completed",
            Self::StockFailed => "stock_failed",
            Self::BlogCreated => "blog_created",
            Self::UserJoined => "user_joined",
            Self::UserLeft => "user_left",
            Self::TorrentProgress => "torrent_progress",
            Self::TorrentCompleted => "torrent_completed",
            Self::TorrentFailed => "torrent_failed",
        }
    }
}
