use crate::{
    errors::AppError,
    repositories::audit_logs,
    structs::{audit_logs::AuditLogQuery, pagination::Paginated},
};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc;

pub use crate::repositories::audit_logs::{AuditEntry, AuditLog};

/// 稽核批次寫入器（啟動時 spawn 一個）。攢批的節奏走 `batch_writer`，與
/// `logging::log_writer` 共用同一份迴圈與常數。
///
/// 為什麼不在 middleware 裡直接 INSERT：audit 掛在 `with_auth` 內層、**不分讀寫**，
/// 每個 `/admin/*` 請求都會產生一筆。原本的寫法是每請求 spawn 一個 task 各自 INSERT，
/// 於是尖峰時稽核寫入會跟真正的查詢搶那 20 條 PG 連線，而稽核完全不需要即時可見。
pub async fn audit_writer(rx: mpsc::Receiver<AuditEntry>, pool: Pool<Postgres>) {
    crate::batch_writer::run(
        rx,
        move |batch| {
            let pool = pool.clone();
            async move { flush(&pool, batch).await }
        },
        // 稽核沒有「丟棄數匯總」這種與資料無關的週期性工作
        || {},
    )
    .await
}

async fn flush(pool: &Pool<Postgres>, buf: Vec<AuditEntry>) {
    // 寫失敗就是這批稽核紀錄遺失，必須留痕（DbLogLayer 會把這則 ERROR 落進 logs 表）
    if let Err(e) = audit_logs::insert_batch(pool, &buf).await {
        tracing::error!("audit_writer 寫入 {} 筆失敗: {:?}", buf.len(), e);
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn get_audit_logs(
    pool: &Pool<Postgres>,
    filter: &AuditLogQuery,
    limit: i64,
    offset: i64,
) -> Result<Paginated<AuditLog>, AppError> {
    // count 與 list 併發跑：序列 await 是白吃一倍延遲（範本同 services/logs.rs）
    let (data, total) = tokio::try_join!(
        audit_logs::get_audit_logs(pool, filter, limit, offset),
        audit_logs::count_audit_logs(pool, filter),
    )?;

    Ok(Paginated::new(data, total))
}
