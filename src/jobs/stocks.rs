use crate::{
    repositories::stocks::{
        get_one_pending_stock_change, get_stock_change_info, update_stock_change_failed,
        upsert_stock_change,
    },
    state::AppStateV2,
    structs::{jobs::AppJob, ws::WsEvent},
};
use async_trait::async_trait;

#[derive(Clone)]
pub struct ConsumePendingStockChangeJob;

#[async_trait]
impl AppJob for ConsumePendingStockChangeJob {
    fn cron_expression(&self) -> &str {
        "0 * * * * *" // 每分鐘執行一次
    }

    async fn run(&self, state: AppStateV2) {
        let pending_stock = match get_one_pending_stock_change(&state).await {
            Ok(Some(stock)) => stock,
            Ok(None) => return,
            Err(err) => {
                tracing::debug!("Error fetching pending stock change: {:?}", err);
                return;
            }
        };

        let stock_info = match get_stock_change_info(&state, &pending_stock).await {
            Ok(info) => info,
            Err(err) => {
                tracing::debug!(
                    "Error fetching stock change info: {:?}, stock_no: {}",
                    err,
                    pending_stock.stock_no
                );
                let _ = update_stock_change_failed(&state, &pending_stock).await;
                state.broadcast(
                    WsEvent::StockFailed,
                    serde_json::json!({ "stock_no": pending_stock.stock_no }),
                );
                return;
            }
        };

        if let Err(err) = upsert_stock_change(&state, &stock_info).await {
            tracing::debug!("Error updating stock_changes: {:?}", err);
            return;
        }

        state.broadcast(
            WsEvent::StockCompleted,
            serde_json::json!({
                "stock_no": stock_info.stock_no,
                "stock_name": stock_info.stock_name,
                "start_date": stock_info.start_date,
                "end_date": stock_info.end_date,
                "change": stock_info.change,
            }),
        );
    }
}
