use axum::response::Json;
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub tx: broadcast::Sender<String>,
    pub redis_pool: RedisPool<RedisConnectionManager>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct DbUser {
    pub id: i64,
    pub email: String,
    pub password: String,
}

impl AppState {
    pub async fn new() -> Self {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");
        let (tx, _rx) = broadcast::channel(64);

        // set up connection pool
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        // redis
        let redis_host = std::env::var("REDIS_HOST").expect("找不到 REDIS_HOST");
        let manager = RedisConnectionManager::new(format!("redis://{}:6379", redis_host)).unwrap();
        let redis_pool = bb8::Pool::builder().build(manager).await.unwrap();
        {
            // ping the database before starting
            let mut conn = redis_pool.get().await.unwrap();
            conn.set::<&str, &str, ()>("foo", "bar").await.unwrap();
            let result: String = conn.get("foo").await.unwrap();
            assert_eq!(result, "bar");
            conn.expire::<&str, ()>("foo", 10).await.unwrap();
        }
        tracing::debug!("successfully connected to redis and pinged it");

        Self {
            pool,
            tx,
            redis_pool,
        }
    }
}

#[derive(Clone)]
pub struct AppStateV2(Arc<AppState>);

impl AppStateV2 {
    pub async fn new() -> Self {
        let app_state = AppState::new().await;
        AppStateV2(Arc::new(app_state))
    }

    pub fn get_pool(&self) -> Pool<Postgres> {
        self.0.pool.clone() // 直接複製
    }

    pub fn get_tx(&self) -> broadcast::Sender<String> {
        self.0.tx.clone() // 直接複製
    }

    // 取 Redis pool
    pub fn get_redis_pool(&self) -> RedisPool<RedisConnectionManager> {
        self.0.redis_pool.clone() // 直接複製
    }

    pub async fn redis_zadd(&self, key: &str, member: &str) -> Result<(), RedisError> {
        let redis_pool = self.get_redis_pool();
        let mut conn = redis_pool.get().await.expect("redis_pool get fail");
        let score = chrono::Utc::now().timestamp_millis();

        conn.zadd(key, member, score).await
    }

    pub async fn redis_zrem(&self, key: &str, members: &str) -> Result<(), RedisError> {
        let redis_pool = self.get_redis_pool();
        let mut conn = redis_pool.get().await.expect("redis_pool get fail");

        conn.zrem(key, members).await
    }

    pub async fn redis_zrange(&self, key: &str) -> Result<Json<Vec<String>>, RedisError> {
        let redis_pool = self.get_redis_pool();
        let mut conn = redis_pool.get().await.expect("redis_pool get fail");

        let result: Vec<String> = conn.zrange(key, 0, -1).await.expect("zrange fail");
        Ok(Json(result))
    }

    pub async fn redis_zrevrange(&self, key: &str) -> Result<Json<Vec<String>>, RedisError> {
        let redis_pool = self.get_redis_pool();
        let mut conn = redis_pool.get().await.expect("redis_pool get fail");

        let result: Vec<String> = conn.zrevrange(key, 0, -1).await.expect("zrevrange fail");
        Ok(Json(result))
    }

    pub async fn check_member_exists(&self, key: &str, member: &str) -> Result<bool, RedisError> {
        let redis_pool = self.get_redis_pool();
        let mut conn = redis_pool.get().await.expect("redis_pool get fail");

        // 使用 zscore 檢查 member 是否存在
        let score: Option<i64> = conn.zscore(key, member).await?;
        Ok(score.is_some()) // 如果 score 為 Some，表示 member 存在；否則為 None，表示不存在
    }

    pub async fn check_email_exists(&self, email: &str) -> Result<DbUser, sqlx::Error> {
        let pool = self.get_pool();

        // 使用 EXISTS 查詢是否有特定 email
        let result: DbUser = sqlx::query_as(
            r#"
                SELECT
                    id,
                    email,
                    password
                FROM
                    users
                WHERE
                    email = $1
                LIMIT
                    1;
            "#,
        )
        .bind(email)
        .fetch_one(&pool)
        .await?;

        Ok(result)
    }

    pub async fn _insert_chat_message(
        &self,
        message_type: &str,
        to_type: &str,
        user_name: &str,
        message: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO chat_messages (message_type, to_type, user_name, message)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(message_type)
        .bind(to_type)
        .bind(user_name)
        .bind(message)
        .execute(&self.get_pool())
        .await?;

        Ok(())
    }

    // 設定 Redis 資料的過期時間（以秒為單位）
    // pub async fn expire_redis_key(&self, key: &str, seconds: usize) -> Result<(), RedisError> {
    //     let app_state = self.0.lock().await;
    //     let mut conn = app_state
    //         .redis_pool
    //         .get()
    //         .await
    //         .expect("get redis_pool fail");
    //     conn.expire(key, seconds as i64).await
    // }
}
