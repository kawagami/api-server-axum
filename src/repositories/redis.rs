use crate::state::AppStateV2;
use axum::response::Json;
use redis::{AsyncCommands, RedisError};

pub async fn redis_zadd(state: &AppStateV2, key: &str, member: &str) -> Result<(), RedisError> {
    let mut conn = state.get_redis_conn().await?;
    let score = chrono::Utc::now().timestamp_millis();

    conn.zadd(key, member, score).await
}

pub async fn redis_zrem(state: &AppStateV2, key: &str, members: &str) -> Result<(), RedisError> {
    let mut conn = state.get_redis_conn().await?;

    conn.zrem(key, members).await
}

pub async fn redis_zrange(state: &AppStateV2, key: &str) -> Result<Json<Vec<String>>, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    Ok(Json(conn.zrange(key, 0, -1).await?))
}

pub async fn _redis_zrevrange(
    state: &AppStateV2,
    key: &str,
) -> Result<Json<Vec<String>>, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    Ok(Json(conn.zrevrange(key, 0, -1).await?))
}

pub async fn check_member_exists(
    state: &AppStateV2,
    key: &str,
    member: &str,
) -> Result<bool, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    // 使用 zscore 檢查 member 是否存在
    let score: Option<i64> = conn.zscore(key, member).await?;
    Ok(score.is_some()) // 如果 score 為 Some，表示 member 存在；否則為 None，表示不存在
}

// 新增函數：設置有效時間 1 小時的鍵值對
pub async fn redis_set(state: &AppStateV2, key: &str, value: &str) -> Result<(), RedisError> {
    let mut conn = state.get_redis_conn().await?;

    conn.set_ex(key, value, 3600).await
}

// 新增函數：檢查 Redis 中的鍵是否存在
pub async fn redis_check_key_exists(state: &AppStateV2, key: &str) -> Result<bool, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    // 使用 EXISTS 命令檢查鍵是否存在 返回 true 表示鍵存在；false 表示鍵不存在
    Ok(conn.exists(key).await?)
}
