use crate::{
    state::AppStateV2,
    structs::{
        jobs::AppJob,
        ws::{ChatMessage, ChatMessageType, To},
    },
};
use async_trait::async_trait;
use chrono::Local; // 引入 chrono 來處理時間

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

        let chat_message = ChatMessage::new_jsonstring(
            ChatMessageType::Message,
            current_time.clone(),
            "KawaBot".to_string(),
            To::All,
        );
        let _ = state.get_tx().send(chat_message);

        // 寫進資料庫 在歷史訊息中會取得
        // let message = format!("{}", current_time);
        // let _ = state
        //     .insert_chat_message("Message", "All", "KawaBot", &message)
        //     .await;
    }
}
