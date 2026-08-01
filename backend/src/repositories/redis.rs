use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, ErrorKind, RedisError};

/// 讀 env 決定 Redis 連線 URL。與 `DATABASE_URL` 對稱：一個完整 URL 就能表達
/// 密碼（`redis://:pw@host`）、TLS（`rediss://`）與 db index，不必為了加密碼改程式碼。
///
/// 前身是 `REDIS_HOST`（host + 6379 寫死在 state.rs）；2026-08-01 換掉，同日
/// VPS 的 kawa.env 也改完，故過渡期的回退路徑已移除。
pub fn redis_url_from_env() -> String {
    env_redis_url().expect("找不到 REDIS_URL（例如 redis://valkey:6379）")
}

/// 空字串視同未設定 —— env 檔裡留著 `REDIS_URL=` 這種半調子的值比沒設更難查
fn env_redis_url() -> Option<String> {
    std::env::var("REDIS_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub async fn get_redis_conn(
    pool: &RedisPool<RedisConnectionManager>,
) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, RedisError> {
    pool.get().await.map_err(|e| match e {
        bb8::RunError::User(err) => err,
        bb8::RunError::TimedOut => {
            RedisError::from((ErrorKind::Io, "Redis connection pool timed out"))
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
    user_id: i64,
    permissions: &[String],
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:permissions:{}", user_id);
    let value = serde_json::to_string(permissions)
        .map_err(crate::errors::AppError::from)?;
    conn.set_ex::<_, _, ()>(key, value, 3600).await?;
    Ok(())
}

pub async fn get_user_permissions(
    pool: &RedisPool<RedisConnectionManager>,
    user_id: i64,
) -> Result<Option<Vec<String>>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:permissions:{}", user_id);
    let value: Option<String> = conn.get(key).await?;
    Ok(value.and_then(|v| serde_json::from_str(&v).ok()))
}

pub async fn del_user_permissions(
    pool: &RedisPool<RedisConnectionManager>,
    user_id: i64,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:permissions:{}", user_id);
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
    user_id: i64,
) {
    if let Err(e) = del_user_permissions(pool, user_id).await {
        tracing::warn!("Failed to invalidate permissions cache for {}: {}", user_id, e);
    }
}

pub async fn invalidate_permissions_for_ids(
    pool: &RedisPool<RedisConnectionManager>,
    ids: &[i64],
) {
    for id in ids {
        invalidate_user_permissions(pool, *id).await;
    }
}

pub async fn del_user_login(
    pool: &RedisPool<RedisConnectionManager>,
    user_id: i64,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("user:login:{}", user_id);
    conn.del::<_, ()>(key).await?;
    Ok(())
}

/// WS 一次性連線票：30 秒 TTL，value 為 admin 顯示名（name）
pub async fn set_ws_ticket(
    pool: &RedisPool<RedisConnectionManager>,
    ticket: &str,
    email: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("ws:ticket:{}", ticket);
    conn.set_ex::<_, _, ()>(key, email, 30).await?;
    Ok(())
}

/// 取出並刪除 WS ticket（一次性），回傳持票人 email；不存在或已用過回 None
pub async fn consume_ws_ticket(
    pool: &RedisPool<RedisConnectionManager>,
    ticket: &str,
) -> Result<Option<String>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let key = format!("ws:ticket:{}", ticket);
    let value: Option<String> = redis::cmd("GETDEL").arg(&key).query_async(&mut *conn).await?;
    Ok(value)
}

pub async fn cache_get(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
) -> Result<Option<String>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let value: Option<String> = conn.get(key).await?;
    Ok(value)
}

pub async fn cache_del(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
) -> Result<(), crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    conn.del::<_, ()>(key).await?;
    Ok(())
}

/// 取出並刪除（一次性消費，GETDEL 原子操作）；不存在回 None。
pub async fn cache_getdel(
    pool: &RedisPool<RedisConnectionManager>,
    key: &str,
) -> Result<Option<String>, crate::errors::AppError> {
    let mut conn = get_redis_conn(pool).await?;
    let value: Option<String> = redis::cmd("GETDEL").arg(key).query_async(&mut *conn).await?;
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

#[cfg(test)]
mod tests {
    //! 針對真實 Redis 的整合測試。
    //!
    //! 為什麼需要：本檔是唯一直接呼叫 redis client 的地方，而它管的是權限快取、
    //! OAuth state、member refresh token 與 WS ticket —— 全是認證路徑。
    //! 升級 redis crate 時「編譯過」完全不代表行為正確（TTL、GETDEL 的一次性、
    //! Option 反序列化等都可能悄悄變樣），故以真實 server 驗證。
    //!
    //! CI 以 valkey service container 提供並設 `REDIS_TEST_REQUIRED=1`；
    //! 本機無 Redis 時自動跳過。設了 REDIS_TEST_REQUIRED 卻連不上會 panic，
    //! 避免 service 掛掉時測試靜默略過而假綠。
    //! 本機要跑：docker run -d -p 6379:6379 valkey/valkey:alpine
    //! 想連別的位址就設 `REDIS_URL`（含 port，不必再映射到 6379）。
    use super::*;
    use bb8::Pool;

    async fn pool() -> Option<Pool<RedisConnectionManager>> {
        let url = env_redis_url().unwrap_or_else(|| "redis://127.0.0.1:6379".into());
        let required = std::env::var("REDIS_TEST_REQUIRED").is_ok();
        let manager = RedisConnectionManager::new(url).expect("build manager");
        let built = Pool::builder()
            .connection_timeout(std::time::Duration::from_secs(3))
            .build(manager)
            .await;
        let p = match built {
            Ok(p) => p,
            Err(e) if required => panic!("REDIS_TEST_REQUIRED 已設定但無法建池: {e}"),
            Err(_) => return None,
        };
        // map 掉連線本身，讓借用在此結束，才能把 pool move 出去
        let probe = p.get().await.map(|_| ());
        match probe {
            Ok(()) => Some(p),
            Err(e) if required => panic!("REDIS_TEST_REQUIRED 已設定但連不上 Redis: {e}"),
            Err(_) => {
                eprintln!("跳過 redis 整合測試：本機無可用 Redis");
                None
            }
        }
    }

    /// 每個測試用獨立 key，避免平行執行互相干擾
    fn uniq(tag: &str) -> String {
        format!("test:{tag}:{}", uuid::Uuid::new_v4())
    }

    #[tokio::test]
    async fn cache_set_get_del_roundtrip() {
        let Some(p) = pool().await else { return };
        let k = uniq("cache");
        assert_eq!(cache_get(&p, &k).await.unwrap(), None, "未寫入前應為 None");
        cache_set(&p, &k, "hello", 60).await.unwrap();
        assert_eq!(cache_get(&p, &k).await.unwrap(), Some("hello".into()));
        cache_del(&p, &k).await.unwrap();
        assert_eq!(cache_get(&p, &k).await.unwrap(), None, "刪除後應為 None");
    }

    #[tokio::test]
    async fn cache_set_applies_ttl() {
        let Some(p) = pool().await else { return };
        let k = uniq("ttl");
        cache_set(&p, &k, "v", 42).await.unwrap();
        let mut conn = get_redis_conn(&p).await.unwrap();
        let ttl: i64 = redis::cmd("TTL")
            .arg(&k)
            .query_async(&mut *conn)
            .await
            .unwrap();
        assert!((1..=42).contains(&ttl), "TTL 應在 1..=42，實際 {ttl}");
        cache_del(&p, &k).await.unwrap();
    }

    #[tokio::test]
    async fn cache_getdel_is_one_shot() {
        let Some(p) = pool().await else { return };
        let k = uniq("getdel");
        cache_set(&p, &k, "once", 60).await.unwrap();
        assert_eq!(cache_getdel(&p, &k).await.unwrap(), Some("once".into()));
        assert_eq!(
            cache_getdel(&p, &k).await.unwrap(),
            None,
            "GETDEL 必須是一次性"
        );
    }

    #[tokio::test]
    async fn ws_ticket_is_one_shot() {
        let Some(p) = pool().await else { return };
        let ticket = uuid::Uuid::new_v4().to_string();
        set_ws_ticket(&p, &ticket, "admin@example.com").await.unwrap();
        assert_eq!(
            consume_ws_ticket(&p, &ticket).await.unwrap(),
            Some("admin@example.com".into())
        );
        assert_eq!(
            consume_ws_ticket(&p, &ticket).await.unwrap(),
            None,
            "WS ticket 必須用過即失效"
        );
    }

    #[tokio::test]
    async fn oauth_state_consumes_exactly_once() {
        let Some(p) = pool().await else { return };
        let s = uuid::Uuid::new_v4().to_string();
        set_oauth_state(&p, &s).await.unwrap();
        assert!(consume_oauth_state(&p, &s).await.unwrap(), "首次消費應成立");
        assert!(!consume_oauth_state(&p, &s).await.unwrap(), "重放應失敗");
        assert!(
            !consume_oauth_state(&p, "never-existed").await.unwrap(),
            "不存在的 state 應回 false"
        );
    }

    #[tokio::test]
    async fn user_permissions_json_roundtrip() {
        let Some(p) = pool().await else { return };
        let uid = 9_000_000 + (rand::random::<u32>() % 100_000) as i64;
        let perms = vec!["blog:update".to_string(), "user:read".to_string()];
        set_user_permissions(&p, uid, &perms).await.unwrap();
        assert_eq!(get_user_permissions(&p, uid).await.unwrap(), Some(perms));
        del_user_permissions(&p, uid).await.unwrap();
        assert_eq!(get_user_permissions(&p, uid).await.unwrap(), None);
    }

    #[tokio::test]
    async fn member_refresh_token_roundtrip() {
        let Some(p) = pool().await else { return };
        let mid = 9_000_000 + (rand::random::<u32>() % 100_000) as i64;
        assert_eq!(get_member_refresh_token(&p, mid).await.unwrap(), None);
        set_member_refresh_token(&p, mid, "jti-abc").await.unwrap();
        assert_eq!(
            get_member_refresh_token(&p, mid).await.unwrap(),
            Some("jti-abc".into())
        );
    }

    #[tokio::test]
    async fn exists_reflects_key_presence() {
        let Some(p) = pool().await else { return };
        let k = uniq("exists");
        assert!(!redis_check_key_exists(&p, &k).await.unwrap());
        redis_set(&p, &k, "x").await.unwrap();
        assert!(redis_check_key_exists(&p, &k).await.unwrap());
        cache_del(&p, &k).await.unwrap();
    }
}
