use crate::{
    errors::AppError,
    repositories::audit_logs,
    structs::{audit_logs::AuditLogQuery, pagination::Paginated},
};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc;

pub use crate::repositories::audit_logs::{AuditEntry, AuditLog};

/// channel 容量。滿了就丟棄新進的稽核紀錄（見 `middleware::audit`）——
/// 稽核不該把請求拖慢，也不該把 DB 連線耗光。
pub const CHANNEL_CAPACITY: usize = 1000;
/// 累積到這個數量就寫一次
const BATCH_SIZE: usize = 50;
/// 沒累積滿也至少這麼久寫一次（毫秒）
const FLUSH_INTERVAL_MS: u64 = 500;

/// 稽核批次寫入器（啟動時 spawn 一個，與 `logging::log_writer` 同形狀）。
///
/// 為什麼不在 middleware 裡直接 INSERT：audit 掛在 `with_auth` 內層、**不分讀寫**，
/// 每個 `/admin/*` 請求都會產生一筆。原本的寫法是每請求 spawn 一個 task 各自 INSERT，
/// 於是尖峰時稽核寫入會跟真正的查詢搶那 20 條 PG 連線，而稽核完全不需要即時可見。
///
/// channel 關閉（正常關機）時把剩下的寫完才結束。
pub async fn audit_writer(mut rx: mpsc::Receiver<AuditEntry>, pool: Pool<Postgres>) {
    let mut buf: Vec<AuditEntry> = Vec::with_capacity(BATCH_SIZE);
    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_millis(FLUSH_INTERVAL_MS));

    loop {
        tokio::select! {
            entry = rx.recv() => {
                match entry {
                    Some(e) => {
                        buf.push(e);
                        if buf.len() >= BATCH_SIZE {
                            flush(&pool, &mut buf).await;
                        }
                    }
                    None => {
                        flush(&pool, &mut buf).await;
                        return;
                    }
                }
            }
            _ = interval.tick() => {
                if !buf.is_empty() {
                    flush(&pool, &mut buf).await;
                }
            }
        }
    }
}

async fn flush(pool: &Pool<Postgres>, buf: &mut Vec<AuditEntry>) {
    if buf.is_empty() {
        return;
    }
    // 寫失敗就是這批稽核紀錄遺失，必須留痕（DbLogLayer 會把這則 ERROR 落進 logs 表）
    if let Err(e) = audit_logs::insert_batch(pool, buf).await {
        tracing::error!("audit_writer 寫入 {} 筆失敗: {:?}", buf.len(), e);
    }
    buf.clear();
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
