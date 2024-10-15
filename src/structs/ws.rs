use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone)]
pub enum To {
    All,
    Private(String), // 這裡的 String 表示特定使用者的 token 或 username
    Myself,
}

#[derive(Serialize)]
pub struct FixedMessageContainer {
    pub buffer: VecDeque<ChatMessage>,
    pub capacity: usize,
}

impl FixedMessageContainer {
    // 初始化一個固定大小的緩衝區
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    // 向緩衝區添加元素
    pub fn add(&mut self, item: ChatMessage) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front(); // 移除最舊的元素
        }
        self.buffer.push_back(item); // 添加新元素
    }

    // 返回緩衝區中的所有元素
    pub fn get_all(&self) -> Vec<&ChatMessage> {
        self.buffer.iter().collect()
    }
}
