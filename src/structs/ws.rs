use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

// 從 DB 取原始資料用的結構
#[derive(Serialize, sqlx::FromRow)]
pub struct DbChatMessage {
    pub id: i32,
    pub message_type: String,
    pub to_type: String,
    pub user_name: String,
    pub message: String,
    pub created_at: DateTime<Utc>, // 對應 TIMESTAMPTZ
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub message_type: ChatMessageType,
    pub content: String,
    pub from: String,
    pub to: To,
    #[serde(default)] // 缺少時使用預設值
    pub created_at: String,
}

impl ChatMessage {
    pub fn new_jsonstring(
        message_type: ChatMessageType,
        content: String,
        from: String,
        to: To,
    ) -> String {
        // 取得目前 UTC 時間
        let now_utc: DateTime<Utc> = Utc::now();

        // 轉換為 UTC+8 時區
        let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
        let now_plus_8 = now_utc.with_timezone(&utc_plus_8);

        // 格式化為 `yyyy-MM-dd HH:mm:ss`
        let now_str = now_plus_8.format("%Y-%m-%d %H:%M:%S").to_string();

        let send_json = ChatMessage {
            message_type,
            content,
            from,
            to,
            created_at: now_str,
        };

        serde_json::to_string(&send_json).expect("產生 json string 失敗")
    }

    pub fn decode(raw_json_string: &str) -> ChatMessage {
        // 從 JSON 字串解析 ChatMessage
        let mut chat_message: ChatMessage =
            serde_json::from_str(raw_json_string).expect("decode raw json string 失敗");

        // 如果 `created_at` 是預設值，動態補充目前時間
        if chat_message.created_at.is_empty() {
            let now_utc: DateTime<Utc> = Utc::now();
            let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
            let now_plus_8 = now_utc.with_timezone(&utc_plus_8);
            chat_message.created_at = now_plus_8.format("%Y-%m-%d %H:%M:%S").to_string();
        }

        chat_message
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ChatMessageType {
    Message,
    Join,
    Leave,
    Info,
    PING,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum To {
    All,
    Private(String), // 這裡的 String 表示特定使用者的 token 或 username
    Myself,
}

#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
}

#[derive(Deserialize)]
pub struct GetParams {
    #[serde(default = "default_limit")]
    pub limit: i32,
}

fn default_limit() -> i32 {
    10
}
