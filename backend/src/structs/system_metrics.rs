use chrono::{DateTime, Utc};
use serde::Serialize;

/// 一筆系統指標快照，同時是 `GET /metrics` 的回應型別。
/// 採集邏輯在 `services::system_metrics`，分桶聚合在 `repositories::system_metrics`。
#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct SystemMetric {
    pub id: i64,
    pub cpu_pct: f32,
    /// hypervisor 抽走 vCPU 的時間佔比,不含在 `cpu_pct` 內(2026-08-08 前兩者混在一起算)
    pub cpu_steal_pct: f32,
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
