use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};

/// 一筆系統指標快照。採集邏輯在 services::system_metrics。
#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct SystemMetric {
    pub id: i64,
    pub cpu_pct: f32,
    pub mem_used_mb: i32,
    pub mem_total_mb: i32,
    pub disk_used_mb: i32,
    pub disk_total_mb: i32,
    pub load1: f32,
    pub load5: f32,
    pub load15: f32,
    /// backend 行程自身 RSS(MB),與整機 mem_used_mb 分開追蹤。
    pub backend_rss_mb: i32,
    pub created_at: DateTime<Utc>,
}

/// 採集當下的量測值(尚未寫入,無 id / created_at)。
pub struct MetricSample {
    pub cpu_pct: f32,
    pub mem_used_mb: i32,
    pub mem_total_mb: i32,
    pub disk_used_mb: i32,
    pub disk_total_mb: i32,
    pub load1: f32,
    pub load5: f32,
    pub load15: f32,
    pub backend_rss_mb: i32,
}

pub async fn insert(pool: &Pool<Postgres>, s: &MetricSample) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO system_metrics
           (cpu_pct, mem_used_mb, mem_total_mb, disk_used_mb, disk_total_mb, load1, load5, load15, backend_rss_mb)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(s.cpu_pct)
    .bind(s.mem_used_mb)
    .bind(s.mem_total_mb)
    .bind(s.disk_used_mb)
    .bind(s.disk_total_mb)
    .bind(s.load1)
    .bind(s.load5)
    .bind(s.load15)
    .bind(s.backend_rss_mb)
    .execute(pool)
    .await?;
    Ok(())
}

/// 取近 N 小時的指標,時間由舊到新(方便前端直接畫折線)。
pub async fn get_recent(pool: &Pool<Postgres>, hours: i64) -> Result<Vec<SystemMetric>, sqlx::Error> {
    sqlx::query_as::<_, SystemMetric>(
        r#"SELECT id, cpu_pct, mem_used_mb, mem_total_mb, disk_used_mb, disk_total_mb,
                  load1, load5, load15, backend_rss_mb, created_at
           FROM system_metrics
           WHERE created_at >= now() - make_interval(hours => $1::int)
           ORDER BY created_at ASC"#,
    )
    .bind(hours)
    .fetch_all(pool)
    .await
}
