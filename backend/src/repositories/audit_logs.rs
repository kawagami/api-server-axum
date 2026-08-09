use crate::structs::audit_logs::AuditLogQuery;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};

#[derive(Serialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: i64,
    /// `admin` / `member` —— 兩種身分的 `user_email` 語意不同，見 `AuditEntry`
    pub actor_type: String,
    pub user_email: String,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub status_code: i16,
    /// 對應 `logs.request_id`，可據此撈出該次請求的完整軌跡（舊資料為 NULL）
    pub request_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 待寫入的一筆稽核紀錄（由 audit middleware 產生，經 channel 交給批次寫入器）。
pub struct AuditEntry {
    /// `admin` 或 `member`。兩者的 id 空間不同（`users.id` vs `members.id`），
    /// 不分開的話「admin 張三」與某個會員可能長得一樣。
    pub actor_type: &'static str,
    /// admin 是顯示名（`users.name`）；member 是 `member#{id}` ——
    /// member 的名字要多打一次 DB 才拿得到，而稽核不值得在請求路徑上加一次查詢。
    pub user_email: String,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub status_code: i16,
    pub request_id: Option<String>,
}

/// 批次寫入。與 `logging.rs::flush` 同形狀（UNNEST 一次 INSERT）：
/// 每請求一個 INSERT 會在流量尖峰時跟正常查詢搶那 20 條連線，而稽核不需要即時可見。
pub async fn insert_batch(
    pool: &Pool<Postgres>,
    entries: &[AuditEntry],
) -> Result<(), sqlx::Error> {
    let actor_types: Vec<&str> = entries.iter().map(|e| e.actor_type).collect();
    let user_emails: Vec<&str> = entries.iter().map(|e| e.user_email.as_str()).collect();
    let methods: Vec<&str> = entries.iter().map(|e| e.method.as_str()).collect();
    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    let queries: Vec<Option<&str>> = entries.iter().map(|e| e.query.as_deref()).collect();
    let status_codes: Vec<i16> = entries.iter().map(|e| e.status_code).collect();
    let request_ids: Vec<Option<&str>> = entries.iter().map(|e| e.request_id.as_deref()).collect();

    sqlx::query(
        "INSERT INTO admin_audit_logs (actor_type, user_email, method, path, query, status_code, request_id)
         SELECT actor_type, user_email, method, path, query, status_code, request_id
         FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::smallint[], $7::text[])
              AS t(actor_type, user_email, method, path, query, status_code, request_id)",
    )
    .bind(&actor_types)
    .bind(&user_emails)
    .bind(&methods)
    .bind(&paths)
    .bind(&queries)
    .bind(&status_codes)
    .bind(&request_ids)
    .execute(pool)
    .await?;

    Ok(())
}

/// list 與 count 共用的 WHERE —— 兩邊漂移會讓 total 與實際筆數對不上
const AUDIT_FILTER: &str = "($1::text IS NULL OR user_email = $1)
             AND ($2::text IS NULL OR method = $2)
             AND ($3::text IS NULL OR path ILIKE '%' || $3 || '%')
             AND ($4::timestamptz IS NULL OR created_at >= $4)
             AND ($5::timestamptz IS NULL OR created_at <= $5)
             AND ($6::text IS NULL OR actor_type = $6)";

pub async fn get_audit_logs(
    pool: &Pool<Postgres>,
    filter: &AuditLogQuery,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLog>, sqlx::Error> {
    sqlx::query_as::<_, AuditLog>(&format!(
        "SELECT id, actor_type, user_email, method, path, query, status_code, request_id, created_at
         FROM admin_audit_logs
         WHERE {AUDIT_FILTER}
         ORDER BY created_at DESC
         LIMIT $7 OFFSET $8"
    ))
    .bind(&filter.user_email)
    .bind(&filter.method)
    .bind(&filter.path)
    .bind(filter.from)
    .bind(filter.to)
    .bind(&filter.actor_type)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_audit_logs(
    pool: &Pool<Postgres>,
    filter: &AuditLogQuery,
) -> Result<i64, sqlx::Error> {
    let (total,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM admin_audit_logs WHERE {AUDIT_FILTER}"
    ))
    .bind(&filter.user_email)
    .bind(&filter.method)
    .bind(&filter.path)
    .bind(filter.from)
    .bind(filter.to)
    .bind(&filter.actor_type)
    .fetch_one(pool)
    .await?;

    Ok(total)
}
