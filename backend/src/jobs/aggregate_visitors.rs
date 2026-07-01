use crate::{repositories::visitors, state::AppState};
use chrono::Duration;

/// 每日台北 00:05（UTC 16:05）：將剛結束的前一台北日的 HLL 計數落地 daily_visitor_stats。
/// 即時當日計數仍直接讀 Redis，不在此處理。
pub async fn run(state: AppState) {
    let yesterday = visitors::taipei_today() - Duration::days(1);

    let count = match visitors::count_day(state.get_redis_pool(), yesterday).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("aggregate_visitors: count_day failed: {}", e);
            return;
        }
    };

    match visitors::upsert_daily(state.get_pool(), yesterday, count).await {
        Ok(_) => tracing::info!("aggregate_visitors: {} unique={}", yesterday, count),
        Err(e) => tracing::error!("aggregate_visitors: upsert failed: {}", e),
    }
}
