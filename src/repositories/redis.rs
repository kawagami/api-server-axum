use crate::state::AppState;
use axum::response::Json;
use redis::{AsyncCommands, RedisError};

pub async fn _redis_zadd(state: &AppState, key: &str, member: &str) -> Result<(), RedisError> {
    let mut conn = state.get_redis_conn().await?;
    let score = chrono::Utc::now().timestamp_millis();

    conn.zadd(key, member, score).await
}

pub async fn _redis_zrem(state: &AppState, key: &str, members: &str) -> Result<(), RedisError> {
    let mut conn = state.get_redis_conn().await?;

    conn.zrem(key, members).await
}

pub async fn _redis_zrange(state: &AppState, key: &str) -> Result<Json<Vec<String>>, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    Ok(Json(conn.zrange(key, 0, -1).await?))
}

pub async fn _redis_zrevrange(
    state: &AppState,
    key: &str,
) -> Result<Json<Vec<String>>, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    Ok(Json(conn.zrevrange(key, 0, -1).await?))
}

pub async fn _check_member_exists(
    state: &AppState,
    key: &str,
    member: &str,
) -> Result<bool, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    // 使用 zscore 檢查 member 是否存在
    let score: Option<i64> = conn.zscore(key, member).await?;
    Ok(score.is_some()) // 如果 score 為 Some，表示 member 存在；否則為 None，表示不存在
}

// 新增函數：設置有效時間 1 小時的鍵值對
pub async fn redis_set(state: &AppState, key: &str, value: &str) -> Result<(), RedisError> {
    let mut conn = state.get_redis_conn().await?;

    conn.set_ex(key, value, 3600).await
}

// 新增函數：檢查 Redis 中的鍵是否存在
pub async fn redis_check_key_exists(state: &AppState, key: &str) -> Result<bool, RedisError> {
    let mut conn = state.get_redis_conn().await?;

    // 使用 EXISTS 命令檢查鍵是否存在 返回 true 表示鍵存在；false 表示鍵不存在
    Ok(conn.exists(key).await?)
}

pub async fn set_user_permissions(
    state: &AppState,
    email: &str,
    permissions: &[String],
) -> Result<(), crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("user:permissions:{}", email);
    let value = serde_json::to_string(permissions)
        .map_err(|e| crate::errors::AppError::from(serde_json::Error::from(e)))?;
    conn.set_ex::<_, _, ()>(key, value, 3600).await?;
    Ok(())
}

pub async fn get_user_permissions(
    state: &AppState,
    email: &str,
) -> Result<Option<Vec<String>>, crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("user:permissions:{}", email);
    let value: Option<String> = conn.get(key).await?;
    Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
}

pub async fn del_user_permissions(
    state: &AppState,
    email: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("user:permissions:{}", email);
    conn.del::<_, ()>(key).await?;
    Ok(())
}
