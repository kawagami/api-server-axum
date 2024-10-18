use serde::{Deserialize, Serialize};

// #[serde_with::serde_as]
#[derive(Serialize, sqlx::FromRow)]
pub struct DbChatMessage {
    pub id: i32,
    pub message_type: String,
    pub to_type: String,
    pub user_name: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub message_type: ChatMessageType,
    pub content: String,
    pub from: String,
    pub to: To,
}

impl ChatMessage {
    pub fn new_jsonstring(
        message_type: ChatMessageType,
        content: String,
        from: String,
        to: To,
    ) -> String {
        let send_json = ChatMessage {
            message_type,
            content,
            from,
            to,
        };
        serde_json::to_string(&send_json).expect("產生 json string 失敗")
    }
    pub fn decode(raw_json_string: &str) -> ChatMessage {
        serde_json::from_str(&raw_json_string).expect("decode raw json string 失敗")
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
