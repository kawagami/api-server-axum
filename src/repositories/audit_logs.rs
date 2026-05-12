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
    pub created_at: DateTime<Utc>,
}

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
        r#"SELECT id, user_email, method, path, query, status_code, created_at
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
