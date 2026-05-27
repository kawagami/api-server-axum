use crate::state::AppState;
use redis::{AsyncCommands, RedisError};

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

pub async fn set_oauth_state(
    state: &AppState,
    state_value: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("oauth:state:{}", state_value);
    conn.set_ex::<_, _, ()>(key, "1", 300).await?;
    Ok(())
}

pub async fn consume_oauth_state(
    state: &AppState,
    state_value: &str,
) -> Result<bool, crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("oauth:state:{}", state_value);
    let deleted: i64 = conn.del(key).await?;
    Ok(deleted > 0)
}

pub async fn set_member_refresh_token(
    state: &AppState,
    member_id: i64,
    jti: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("member:refresh:{}", member_id);
    conn.set_ex::<_, _, ()>(key, jti, 30 * 24 * 3600).await?;
    Ok(())
}

pub async fn get_member_refresh_token(
    state: &AppState,
    member_id: i64,
) -> Result<Option<String>, crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("member:refresh:{}", member_id);
    Ok(conn.get(key).await?)
}

pub async fn invalidate_permissions_for_emails(state: &AppState, emails: &[String]) {
    for email in emails {
        let _ = del_user_permissions(state, email).await;
    }
}

pub async fn del_user_login(
    state: &AppState,
    email: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("user:login:{}", email);
    conn.del::<_, ()>(key).await?;
    Ok(())
}

pub async fn _del_member_refresh_token(
    state: &AppState,
    member_id: i64,
) -> Result<(), crate::errors::AppError> {
    let mut conn = state.get_redis_conn().await?;
    let key = format!("member:refresh:{}", member_id);
    conn.del::<_, ()>(key).await?;
    Ok(())
}
