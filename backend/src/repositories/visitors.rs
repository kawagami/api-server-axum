use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use chrono::{Datelike, FixedOffset, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};

use super::redis::get_redis_conn;

/// HLL key 保留天數（過後 Redis 自動清；歷史已落 daily_visitor_stats）
const KEY_TTL_SECS: i64 = 40 * 24 * 3600;

/// 以台北時間（UTC+8）取「今天」的日期
pub fn taipei_today() -> NaiveDate {
    let tz = FixedOffset::east_opt(8 * 3600).unwrap();
    Utc::now().with_timezone(&tz).date_naive()
}

fn day_key(date: NaiveDate) -> String {
    format!("visitors:{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

/// 記一次到訪：以 hash(ip|ua) 為去重元素 PFADD 進當日 HLL，並確保 key 有 TTL。
/// best-effort，失敗只記 warn 不影響連線。
pub async fn record_visit(
    pool: &RedisPool<RedisConnectionManager>,
    ip: &str,
    user_agent: &str,
) {
    if let Err(e) = record_visit_inner(pool, ip, user_agent).await {
        tracing::warn!("record_visit failed: {}", e);
    }
}

async fn record_visit_inner(
    pool: &RedisPool<RedisConnectionManager>,
    ip: &str,
    user_agent: &str,
) -> Result<(), redis::RedisError> {
    let key = day_key(taipei_today());
    let element = format!("{ip}|{user_agent}");
    let mut conn = get_redis_conn(pool).await?;
    // PFADD 後若 key 尚無 TTL（當日首寫）才設定，避免每次到訪都重設效期
    let _: i64 = redis::Script::new(
        "redis.call('PFADD', KEYS[1], ARGV[1]) \
         if redis.call('TTL', KEYS[1]) < 0 then redis.call('EXPIRE', KEYS[1], ARGV[2]) end \
         return 1",
    )
    .key(&key)
    .arg(element)
    .arg(KEY_TTL_SECS)
    .invoke_async(&mut *conn)
    .await?;
    Ok(())
}

/// 單日不重複到訪（HLL 近似值）
pub async fn count_day(
    pool: &RedisPool<RedisConnectionManager>,
    date: NaiveDate,
) -> Result<i64, redis::RedisError> {
    let mut conn = get_redis_conn(pool).await?;
    let count: i64 = redis::cmd("PFCOUNT")
        .arg(day_key(date))
        .query_async(&mut *conn)
        .await?;
    Ok(count)
}

/// 多日合併後的不重複到訪（PFCOUNT 多 key 原生 merge，跨日去重）
pub async fn count_days(
    pool: &RedisPool<RedisConnectionManager>,
    dates: &[NaiveDate],
) -> Result<i64, redis::RedisError> {
    if dates.is_empty() {
        return Ok(0);
    }
    let mut conn = get_redis_conn(pool).await?;
    let mut cmd = redis::cmd("PFCOUNT");
    for d in dates {
        cmd.arg(day_key(*d));
    }
    let count: i64 = cmd.query_async(&mut *conn).await?;
    Ok(count)
}

/// 將某日 HLL 計數落地（每日 job 用）；同日重跑覆蓋
pub async fn upsert_daily(
    pool: &Pool<Postgres>,
    date: NaiveDate,
    unique_visitors: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO daily_visitor_stats (date, unique_visitors)
           VALUES ($1, $2)
           ON CONFLICT (date) DO UPDATE SET unique_visitors = EXCLUDED.unique_visitors"#,
    )
    .bind(date)
    .bind(unique_visitors)
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Serialize, sqlx::FromRow)]
pub struct DailyVisitorStat {
    pub date: NaiveDate,
    pub unique_visitors: i64,
}

/// 歷史每日不重複到訪，近 limit 天，新到舊
pub async fn history(
    pool: &Pool<Postgres>,
    limit: i64,
) -> Result<Vec<DailyVisitorStat>, sqlx::Error> {
    sqlx::query_as::<_, DailyVisitorStat>(
        r#"SELECT date, unique_visitors
           FROM daily_visitor_stats
           ORDER BY date DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}
