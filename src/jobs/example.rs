use crate::{
    state::AppStateV2,
    structs::{
        jobs::AppJob,
        ws::{ChatMessage, ChatMessageType, To},
    },
};
use async_trait::async_trait;
use chrono::Local;
use tracing::{error, info};

#[derive(Clone)]
pub struct ExampleJob;

#[async_trait]
impl AppJob for ExampleJob {
    fn cron_expression(&self) -> &str {
        "0 * * * * *" // 每分鐘執行一次
    }

    async fn run(&self, state: AppStateV2) {
        // 獲取當前 UTC+8 時間並格式化
        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        info!("執行定時任務: 發送當前時間 {}", current_time);

        // 創建聊天訊息
        let chat_message = ChatMessage::new(
            None,
            ChatMessageType::Message,
            current_time,
            "KawaBot".to_string(),
            To::All,
        );

        // 序列化為 JSON 字串
        match serde_json::to_string(&chat_message) {
            Ok(json_message) => {
                if let Err(err) = state.get_tx().send(json_message) {
                    error!("廣播定時訊息失敗: {}", err);
                }
            }
            Err(err) => {
                error!("序列化聊天訊息失敗: {}", err);
            }
        }
    }
}
