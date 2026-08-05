use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};

#[derive(Serialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: i64,
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
    /// 操作者顯示名（`users.name`）
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
    let user_emails: Vec<&str> = entries.iter().map(|e| e.user_email.as_str()).collect();
    let methods: Vec<&str> = entries.iter().map(|e| e.method.as_str()).collect();
    let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
    let queries: Vec<Option<&str>> = entries.iter().map(|e| e.query.as_deref()).collect();
    let status_codes: Vec<i16> = entries.iter().map(|e| e.status_code).collect();
    let request_ids: Vec<Option<&str>> = entries.iter().map(|e| e.request_id.as_deref()).collect();

    sqlx::query(
        "INSERT INTO admin_audit_logs (user_email, method, path, query, status_code, request_id)
         SELECT user_email, method, path, query, status_code, request_id
         FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::smallint[], $6::text[])
              AS t(user_email, method, path, query, status_code, request_id)",
    )
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

// 篩選參數各自獨立可省略，收成 struct 反而多一層；維持平鋪
#[allow(clippy::too_many_arguments)]
pub async fn get_audit_logs(
    pool: &Pool<Postgres>,
    user_email: Option<String>,
    method: Option<String>,
    path_contains: Option<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: i64,
    offset: i64,
) -> Result<Vec<AuditLog>, sqlx::Error> {
    sqlx::query_as::<_, AuditLog>(
        r#"SELECT id, user_email, method, path, query, status_code, request_id, created_at
           FROM admin_audit_logs
           WHERE ($1::text IS NULL OR user_email = $1)
             AND ($2::text IS NULL OR method = $2)
             AND ($3::text IS NULL OR path ILIKE '%' || $3 || '%')
             AND ($4::timestamptz IS NULL OR created_at >= $4)
             AND ($5::timestamptz IS NULL OR created_at <= $5)
           ORDER BY created_at DESC
           LIMIT $6 OFFSET $7"#,
    )
    .bind(user_email)
    .bind(method)
    .bind(path_contains)
    .bind(from)
    .bind(to)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
