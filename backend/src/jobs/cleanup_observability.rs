use crate::state::AppState;

/// 每日清理觀測資料,避免在小磁碟上無限成長:
/// - logs 保留 14 天(只存 WARN/ERROR,量本就不大)
/// - system_metrics 保留 90 天(每分鐘一筆)
pub async fn run(state: AppState) {
    let pool = state.get_pool();

    match sqlx::query("DELETE FROM logs WHERE created_at < now() - interval '14 days'")
        .execute(pool)
        .await
    {
        Ok(r) => tracing::info!("cleanup_observability: logs deleted {}", r.rows_affected()),
        Err(e) => tracing::error!("cleanup_observability: logs delete failed: {e}"),
    }

    match sqlx::query("DELETE FROM system_metrics WHERE created_at < now() - interval '90 days'")
        .execute(pool)
        .await
    {
        Ok(r) => tracing::info!("cleanup_observability: metrics deleted {}", r.rows_affected()),
        Err(e) => tracing::error!("cleanup_observability: metrics delete failed: {e}"),
    }
}
