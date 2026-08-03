use crate::structs::logs::LogQuery;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::{Pool, Postgres};

#[derive(Serialize, sqlx::FromRow)]
pub struct Log {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<i32>,
    /// 對應 `x-request-id` / 錯誤 body 的 `request_id`，用來把一個請求的 log 串起來
    pub request_id: Option<String>,
    /// event 與 span 的其餘 field（`self` = 錯誤細節、`method`、`path`…），見 `logging.rs`
    pub fields: Option<Value>,
    pub created_at: DateTime<Utc>,
}

/// list 與 by_request 共用（欄位清單抄兩份就是下一個會長歪的地方）
const LOG_COLUMNS: &str = "id, level, message, target, file, line, request_id, fields, created_at";

/// list 與 count 共用的篩選條件。**兩邊的 bind 順序必須一致**（$1..$6），
/// 加參數要同時改 `get_logs` 與 `count_logs` 的 bind ——
/// 條件寫兩份就是 total 與 data 對不上的來源（範本同 `repositories/vocab.rs::ADMIN_FILTER`）。
///
/// `q` 一併掃 `fields::text`：錯誤細節現在存在 fields 裡（`?self`），只搜 message
/// 會搜不到有用的東西。這張表只收 WARN+ 故量小，無索引的 ILIKE 可接受；
/// 真的變慢就先帶 from/to 縮範圍（`created_at` 有索引）。
const LOG_FILTER: &str = "($1::text[] IS NULL OR level = ANY($1))
             AND ($2::text IS NULL OR message ILIKE '%' || $2 || '%'
                                   OR fields::text ILIKE '%' || $2 || '%')
             AND ($3::text IS NULL OR target ILIKE '%' || $3 || '%')
             AND ($4::text IS NULL OR request_id = $4)
             AND ($5::timestamptz IS NULL OR created_at >= $5)
             AND ($6::timestamptz IS NULL OR created_at <= $6)";

/// 單一請求最多回這麼多筆。正常請求個位數，設上限只為擋異常暴量把回應撐爆。
const REQUEST_TRACE_LIMIT: i64 = 500;

pub async fn get_logs(
    pool: &Pool<Postgres>,
    filter: &LogQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<Log>, sqlx::Error> {
    sqlx::query_as::<_, Log>(&format!(
        "SELECT {LOG_COLUMNS}
         FROM logs
         WHERE {LOG_FILTER}
         ORDER BY created_at DESC, id DESC
         LIMIT $7 OFFSET $8"
    ))
    .bind(filter.levels())
    .bind(&filter.q)
    .bind(&filter.target)
    .bind(&filter.request_id)
    .bind(filter.from)
    .bind(filter.to)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_logs(pool: &Pool<Postgres>, filter: &LogQuery) -> Result<i64, sqlx::Error> {
    let (total,): (i64,) =
        sqlx::query_as(&format!("SELECT COUNT(*) FROM logs WHERE {LOG_FILTER}"))
            .bind(filter.levels())
            .bind(&filter.q)
            .bind(&filter.target)
            .bind(&filter.request_id)
            .bind(filter.from)
            .bind(filter.to)
            .fetch_one(pool)
            .await?;

    Ok(total)
}

/// 單一請求的完整軌跡。**時間正序**（照發生順序讀），與列表的新到舊刻意相反。
pub async fn logs_by_request(
    pool: &Pool<Postgres>,
    request_id: &str,
) -> Result<Vec<Log>, sqlx::Error> {
    sqlx::query_as::<_, Log>(&format!(
        "SELECT {LOG_COLUMNS}
         FROM logs
         WHERE request_id = $1
         ORDER BY created_at ASC, id ASC
         LIMIT $2"
    ))
    .bind(request_id)
    .bind(REQUEST_TRACE_LIMIT)
    .fetch_all(pool)
    .await
}
