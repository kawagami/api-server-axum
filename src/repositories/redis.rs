use crate::state::AppStateV2;
use axum::response::Json;
use redis::{AsyncCommands, RedisError};

pub async fn redis_zadd(state: &AppStateV2, key: &str, member: &str) -> Result<(), RedisError> {
    let redis_pool = state.get_redis_pool();
    let mut conn = redis_pool.get().await.expect("redis_pool get fail");
    let score = chrono::Utc::now().timestamp_millis();

    conn.zadd(key, member, score).await
}

pub async fn redis_zrem(state: &AppStateV2, key: &str, members: &str) -> Result<(), RedisError> {
    let redis_pool = state.get_redis_pool();
    let mut conn = redis_pool.get().await.expect("redis_pool get fail");

    conn.zrem(key, members).await
}

pub async fn redis_zrange(state: &AppStateV2, key: &str) -> Result<Json<Vec<String>>, RedisError> {
    let redis_pool = state.get_redis_pool();
    let mut conn = redis_pool.get().await.expect("redis_pool get fail");

    let result: Vec<String> = conn.zrange(key, 0, -1).await.expect("zrange fail");
    Ok(Json(result))
}

pub async fn _redis_zrevrange(
    state: &AppStateV2,
    key: &str,
) -> Result<Json<Vec<String>>, RedisError> {
    let redis_pool = state.get_redis_pool();
    let mut conn = redis_pool.get().await.expect("redis_pool get fail");

    let result: Vec<String> = conn.zrevrange(key, 0, -1).await.expect("zrevrange fail");
    Ok(Json(result))
}

pub async fn check_member_exists(
    state: &AppStateV2,
    key: &str,
    member: &str,
) -> Result<bool, RedisError> {
    let redis_pool = state.get_redis_pool();
    let mut conn = redis_pool.get().await.expect("redis_pool get fail");

    // 使用 zscore 檢查 member 是否存在
    let score: Option<i64> = conn.zscore(key, member).await?;
    Ok(score.is_some()) // 如果 score 為 Some，表示 member 存在；否則為 None，表示不存在
}
