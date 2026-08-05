use crate::{errors::AppError, repositories::visitors, state::AppState};
use chrono::{Duration, NaiveDate};

/// 每日台北 00:05（UTC 16:05）：將剛結束的前一台北日的 HLL 計數落地 daily_visitor_stats。
/// 即時當日計數仍直接讀 Redis，不在此處理。
pub async fn run(state: AppState) {
    let yesterday = crate::utils::date::taipei_today() - Duration::days(1);

    // 這個 job 沒有自癒能力：算的是「前一台北日」，下一輪跑的是隔天的區間、不會回頭補。
    // 當下 Redis 或 DB 不通就是那天永久空洞，所以要重試（Redis 的 HLL key 有 40 天 TTL，
    // 重試期間資料還在；upsert_daily 本身是 upsert，重試安全）。
    super::run_with_retries(
        "aggregate_visitors",
        3,
        std::time::Duration::from_secs(3600),
        || aggregate(&state, yesterday),
    )
    .await;
}

async fn aggregate(state: &AppState, day: NaiveDate) -> Result<(), AppError> {
    let count = visitors::count_day(state.get_redis_pool(), day).await?;
    visitors::upsert_daily(state.get_pool(), day, count).await?;
    tracing::info!("aggregate_visitors: {} unique={}", day, count);
    Ok(())
}
