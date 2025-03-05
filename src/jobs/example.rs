use crate::{
    state::AppStateV2,
    structs::{
        jobs::AppJob,
        ws::{ChatMessage, ChatMessageType, To},
    },
};
use async_trait::async_trait;
use chrono::Local;

#[derive(Clone)]
pub struct ExampleJob;

#[async_trait]
impl AppJob for ExampleJob {
    fn cron_expression(&self) -> &str {
        "0 * * * * *" // 每分鐘執行一次
    }

    async fn run(&self, state: AppStateV2) {
        // 創建聊天訊息
        match ChatMessage::new(
            None,
            ChatMessageType::Message,
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "KawaBot".to_string(),
            To::All,
        )
        .to_json_string()
        {
            Ok(json_message) => {
                let _ = state.get_tx().send(json_message);
            }
            Err(err) => {
                tracing::error!("序列化聊天訊息失敗: {}", err);
            }
        }
    }
}
