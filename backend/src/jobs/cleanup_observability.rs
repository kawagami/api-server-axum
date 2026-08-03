use crate::state::AppState;

/// 每日清理觀測資料,避免在小磁碟上無限成長:
/// - logs 保留 14 天(只存 WARN/ERROR,量本就不大)
/// - system_metrics 保留 90 天(每分鐘一筆)
/// - admin_audit_logs 保留 180 天
///
/// 稽核的保留期刻意比另兩者長 —— 那是「誰做了什麼」的問責軌跡,回溯需求本來就比日誌久。
/// 加這條的動因:audit middleware **不分讀寫、GET 也記**(見 `middleware/audit.rs`),
/// 所以任何唯讀查詢(含 `scripts/kawa-logs`)都會累積筆數,而這張表原本是全站唯一
/// 沒有上限的觀測表。2026-08-03 導入時實測 7228 筆 / 最舊 83 天,故當下刪 0 筆,
/// 是純未來防護而非一次性清理。
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

    match sqlx::query(
        "DELETE FROM admin_audit_logs WHERE created_at < now() - interval '180 days'",
    )
    .execute(pool)
    .await
    {
        Ok(r) => tracing::info!("cleanup_observability: audit deleted {}", r.rows_affected()),
        Err(e) => tracing::error!("cleanup_observability: audit delete failed: {e}"),
    }
}
