use std::collections::VecDeque;

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct WsMessage {
    pub message: String,
    pub from: String,
}

#[derive(Serialize)]
pub struct FixedMessageContainer {
    pub buffer: VecDeque<WsMessage>,
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
    pub fn add(&mut self, item: WsMessage) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front(); // 移除最舊的元素
        }
        self.buffer.push_back(item); // 添加新元素
    }

    // 返回緩衝區中的所有元素
    pub fn get_all(&self) -> Vec<&WsMessage> {
        self.buffer.iter().collect()
    }
}
