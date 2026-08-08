use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};

/// 一筆系統指標快照。採集邏輯在 services::system_metrics。
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

/// 採集當下的量測值(尚未寫入,無 id / created_at)。
pub struct MetricSample {
    pub cpu_pct: f32,
    pub cpu_steal_pct: f32,
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
           (cpu_pct, cpu_steal_pct, mem_used_mb, mem_total_mb, disk_used_mb, disk_total_mb, load1, load5, load15, backend_rss_mb)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
    )
    .bind(s.cpu_pct)
    .bind(s.cpu_steal_pct)
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

/// 前端折線圖寬約 720px,每點小於 1px 就看不出差別;
/// 依查詢範圍把每分鐘一筆的原始採樣聚成約 720 個時間桶,避免長範圍一次吐上萬筆
/// (168 小時原本 10080 筆 ≈ 2MB JSON,還要在瀏覽器畫上萬個 SVG 節點)。
///
/// 回傳桶寬秒數,永遠是整分鐘且不小於採樣間隔(再細沒有意義,聚合會退化成原始資料)。
pub fn bucket_seconds(hours: i64) -> i64 {
    const TARGET_POINTS: i64 = 720;
    const SAMPLE_SECONDS: i64 = 60; // collect_system_metrics 每分鐘跑一次

    let raw = (hours * 3600 / TARGET_POINTS).max(SAMPLE_SECONDS);
    (raw + 59) / 60 * 60 // 無條件進位到整分鐘
}

/// 取近 N 小時的指標,時間由舊到新(方便前端直接畫折線)。
///
/// 依 [`bucket_seconds`] 聚合成時間桶,`created_at` 是桶的起點。
/// 桶內取 **max** 而非 avg —— 這頁是拿來找尖峰的,平均會把短暫的 CPU/load 爆衝抹平。
/// 每筆原始採樣本身已是整個採樣間隔(1 分鐘)的平均,所以這裡的 max = 該桶內最忙的那一分鐘。
/// 12 小時以內桶寬會退化成 60 秒,等同原始每分鐘採樣。
pub async fn get_recent(pool: &Pool<Postgres>, hours: i64) -> Result<Vec<SystemMetric>, sqlx::Error> {
    sqlx::query_as::<_, SystemMetric>(
        r#"SELECT max(id) AS id,
                  max(cpu_pct) AS cpu_pct,
                  max(cpu_steal_pct) AS cpu_steal_pct,
                  max(mem_used_mb) AS mem_used_mb,
                  max(mem_total_mb) AS mem_total_mb,
                  max(disk_used_mb) AS disk_used_mb,
                  max(disk_total_mb) AS disk_total_mb,
                  max(load1) AS load1,
                  max(load5) AS load5,
                  max(load15) AS load15,
                  max(backend_rss_mb) AS backend_rss_mb,
                  date_bin(make_interval(secs => $2::int), created_at, timestamptz 'epoch') AS created_at
           FROM system_metrics
           WHERE created_at >= now() - make_interval(hours => $1::int)
           -- 用輸出欄位序號而非別名:別名 created_at 與來源欄位同名,
           -- PG 遇到 GROUP BY 名稱歧義時會選「輸入欄位」,那樣等於沒分桶。
           -- ⚠ 序號 = date_bin 那欄的位置,SELECT 加欄位要同步改這兩個數字
           GROUP BY 12
           ORDER BY 12 ASC"#,
    )
    .bind(hours)
    .bind(bucket_seconds(hours))
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::bucket_seconds;

    #[test]
    fn short_ranges_keep_raw_one_minute_samples() {
        // 12 小時以內,720 點裝得下每分鐘一筆,不該被聚合掉
        assert_eq!(bucket_seconds(1), 60);
        assert_eq!(bucket_seconds(12), 60);
    }

    #[test]
    fn long_ranges_bucket_to_about_720_points() {
        for hours in [24, 72, 168] {
            let secs = bucket_seconds(hours);
            let points = hours * 3600 / secs;
            assert!(
                (600..=800).contains(&points),
                "hours={hours} secs={secs} points={points}"
            );
        }
        assert_eq!(bucket_seconds(24), 120);
        assert_eq!(bucket_seconds(72), 360);
        assert_eq!(bucket_seconds(168), 840);
    }

    #[test]
    fn always_whole_minutes() {
        // 桶寬非整分鐘會讓 X 軸時間標籤跳得很醜
        for hours in 1..=168 {
            assert_eq!(bucket_seconds(hours) % 60, 0, "hours={hours}");
        }
    }
}
