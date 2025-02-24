use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatMessageType {
    Message,
    Join,
    Leave,
    System,
    PING,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum To {
    All,
    Private(String),
    Myself,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub message_type: ChatMessageType,
    pub content: String,
    pub from: String,
    pub to: To,
    #[serde(default = "default_created_at")]
    pub created_at: String,
}

fn default_created_at() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

impl ChatMessage {
    pub fn new(
        id: Option<i64>,
        message_type: ChatMessageType,
        content: String,
        from: String,
        to: To,
    ) -> Self {
        ChatMessage {
            id,
            message_type,
            content,
            from,
            to,
            created_at: default_created_at(),
        }
    }

    pub fn decode(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    // 新增方法：將 self 轉換成 JSON 字串
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}

impl From<ChatMessage> for String {
    fn from(msg: ChatMessage) -> Self {
        msg.to_string()
    }
}
#[derive(Deserialize)]
pub struct QueryParams {
    pub token: String,
}

#[derive(Deserialize)]
pub struct GetParams {
    #[serde(default = "default_limit")]
    pub limit: i32,
    pub before_id: Option<i32>,
}

fn default_limit() -> i32 {
    10
}
