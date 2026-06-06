use crate::{state::AppState, structs::jobs::AppJob};
use async_trait::async_trait;

pub struct CleanupStockChangeDuplicatesJob;

#[async_trait]
impl AppJob for CleanupStockChangeDuplicatesJob {
    fn cron_expression(&self) -> &str {
        "0 0 21 * * *" // UTC 21:00，FetchBuybackPeriodsJob (20:00) 跑完後
    }

    async fn run(&self, state: AppState) {
        let result = sqlx::query(
            r#"
            DELETE FROM stock_changes
            WHERE id IN (
                SELECT sc.id
                FROM stock_changes sc
                JOIN stock_buyback_periods bp
                    ON bp.stock_no = sc.stock_no
                    AND bp.start_date = sc.start_date
                    AND bp.end_date != sc.end_date
                WHERE EXISTS (
                    SELECT 1 FROM stock_changes sc2
                    WHERE sc2.stock_no = sc.stock_no
                      AND sc2.start_date = sc.start_date
                      AND sc2.end_date = bp.end_date
                )
            )
            "#,
        )
        .execute(state.get_pool())
        .await;

        match result {
            Ok(r) if r.rows_affected() > 0 => {
                tracing::info!("cleanup_stock_change_duplicates: deleted {} stale records", r.rows_affected());
            }
            Ok(_) => {}
            Err(e) => tracing::error!("cleanup_stock_change_duplicates fail: {}", e),
        }
    }
}
