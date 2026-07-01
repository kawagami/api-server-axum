use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};

#[derive(Serialize, sqlx::FromRow)]
pub struct Log {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<i32>,
    pub created_at: DateTime<Utc>,
}

pub async fn get_logs(
    pool: &Pool<Postgres>,
    level: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Log>, sqlx::Error> {
    sqlx::query_as::<_, Log>(
        r#"SELECT id, level, message, target, file, line, created_at
           FROM logs
           WHERE ($1::text IS NULL OR level = $1)
           ORDER BY created_at DESC
           LIMIT $2 OFFSET $3"#,
    )
    .bind(level)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
