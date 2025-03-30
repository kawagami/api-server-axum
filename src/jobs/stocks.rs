use crate::{
    repositories::stocks::{
        get_one_pending_stock_change, get_stock_change_info, update_stock_change_failed,
    },
    state::AppStateV2,
    structs::jobs::AppJob,
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
        // 取排程中的任務
        let pending_stock = match get_one_pending_stock_change(&state).await {
            Ok(Some(stock)) => stock,
            Ok(None) => return, // 沒有任務，直接返回
            Err(err) => {
                tracing::debug!("Error fetching pending stock change: {:?}", err);
                return;
            }
        };

        // 根據 stock_no, start_date, end_date 取得股票變動資訊
        let stock_info = match get_stock_change_info(&state, &pending_stock).await {
            Ok(info) => info,
            Err(err) => {
                tracing::debug!(
                    "Error fetching stock change info: {:?}, stock_no: {}",
                    err,
                    pending_stock.stock_no
                );
                let _ = update_stock_change_failed(&state, &pending_stock).await;
                return;
            }
        };

        // 更新或插入數據
        if let Err(err) = sqlx::query(
            r#"
            INSERT INTO stock_changes (
                stock_no,
                stock_name,
                start_date,
                start_price,
                end_date,
                end_price,
                change,
                status,
                created_at,
                updated_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, 'completed', now(), now()
            )
            ON CONFLICT (stock_no, start_date, end_date) 
            DO UPDATE SET
                status = 'completed',
                stock_name = EXCLUDED.stock_name,
                start_price = EXCLUDED.start_price,
                end_price = EXCLUDED.end_price,
                change = EXCLUDED.change,
                updated_at = now()
            "#,
        )
        .bind(&stock_info.stock_no)
        .bind(&stock_info.stock_name)
        .bind(&stock_info.start_date)
        .bind(&stock_info.start_price)
        .bind(&stock_info.end_date)
        .bind(&stock_info.end_price)
        .bind(&stock_info.change)
        .execute(state.get_pool())
        .await
        {
            tracing::debug!("Error updating stock_changes: {:?}", err);
        }
    }
}
