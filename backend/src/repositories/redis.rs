use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, ErrorKind, RedisError};

pub async fn get_redis_conn(
    pool: &RedisPool<RedisConnectionManager>,
) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, RedisError> {
    pool.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => err,
        bb8::RunError::TimedOut => {
            RedisError::from((ErrorKind::IoError, "Redis connection pool timed out"))
        }
    })
}

pub async fn redis_set(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
    value: &str,
) -> Result<(), RedisError> {
    let mut conn = get_redis_conn(pool).await?;
    conn.set_ex(key, value, 3600).await
}

pub async fn redis_check_key_exists(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
) -> Result<bool, RedisError> {
    let mut conn = get_redis_conn(pool).await?;
    conn.exists(key).await
}

pub async fn set_user_permissions(
    pool: &RedisPool<RedisConnectionManager>,
    email: &str,
    permissions: &[String],
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:permissions:{}", email);
    let value = serde_json::to_string(permissions)
        .map_err(crate::errors::AppError::from)?;
    conn.set_ex::<_, _, ()>(key, value, 3600).await?;
    Ok(())
}

pub async fn get_user_permissions(
    pool: &RedisPool<RedisConnectionManager>,
    email: &str,
) -> Result<Option<Vec<String>>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:permissions:{}", email);
    let value: Option<String> = conn.get(key).await?;
    Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
}

pub async fn del_user_permissions(
    pool: &RedisPool<RedisConnectionManager>,
    email: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:permissions:{}", email);
    conn.del::<_, ()>(key).await?;
    Ok(())
}

pub async fn set_oauth_state(
    pool: &RedisPool<RedisConnectionManager>,
    state_value: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("oauth:state:{}", state_value);
    conn.set_ex::<_, _, ()>(key, "1", 300).await?;
    Ok(())
}

pub async fn consume_oauth_state(
    pool: &RedisPool<RedisConnectionManager>,
    state_value: &str,
) -> Result<bool, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("oauth:state:{}", state_value);
    let deleted: i64 = conn.del(key).await?;
    Ok(deleted > 0)
}

pub async fn set_member_refresh_token(
    pool: &RedisPool<RedisConnectionManager>,
    member_id: i64,
    jti: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("member:refresh:{}", member_id);
    conn.set_ex::<_, _, ()>(key, jti, 30 * 24 * 3600).await?;
    Ok(())
}

pub async fn get_member_refresh_token(
    pool: &RedisPool<RedisConnectionManager>,
    member_id: i64,
) -> Result<Option<String>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("member:refresh:{}", member_id);
    Ok(conn.get(key).await?)
}

/// 失效單一 user 的權限快取 — 失敗只記 warn，不阻斷主流程
pub async fn invalidate_user_permissions(
    pool: &RedisPool<RedisConnectionManager>,
    email: &str,
) {
    if let Err(e) = del_user_permissions(pool, email).await {
        tracing::warn!("Failed to invalidate permissions cache for {}: {}", email, e);
    }
}

pub async fn invalidate_permissions_for_emails(
    pool: &RedisPool<RedisConnectionManager>,
    emails: &[String],
) {
    for email in emails {
        invalidate_user_permissions(pool, email).await;
    }
}

pub async fn del_user_login(
    pool: &RedisPool<RedisConnectionManager>,
    email: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:login:{}", email);
    conn.del::<_, ()>(key).await?;
    Ok(())
}

pub async fn cache_get(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
) -> Result<Option<String>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let value: Option<String> = conn.get(key).await?;
    Ok(value)
}

pub async fn cache_set(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
    value: &str,
    ttl_secs: u64,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    conn.set_ex::<_, _, ()>(key, value, ttl_secs).await?;
    Ok(())
}
