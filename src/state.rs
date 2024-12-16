use crate::structs::{blogs::DbBlog, hackmd::Post};
use axum::response::Json;
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, QueryBuilder};
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub tx: broadcast::Sender<String>,
    pub redis_pool: RedisPool<RedisConnectionManager>,
    pub http_client: Client, // 新增 reqwest::Client
    pub fastapi_upload_host: String,
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

        // 初始化 HTTP 客戶端
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10)) // 設定超時時間
            .build()
            .expect("Failed to build HTTP client");

        // FastAPI upload host
        let fastapi_upload_host =
            std::env::var("FASTAPI_UPLOAD_HOST").expect("找不到 FASTAPI_UPLOAD_HOST");

        Self {
            pool,
            tx,
            redis_pool,
            http_client,
            fastapi_upload_host,
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

    pub fn get_http_client(&self) -> Client {
        self.0.http_client.clone()
    }

    pub fn get_fastapi_upload_host(&self) -> &str {
        &self.0.fastapi_upload_host
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

    pub async fn insert_chat_message(
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

    pub async fn delete_posts(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM hackmd_posts;")
            .execute(&self.get_pool())
            .await?;

        Ok(())
    }

    // bulk insert
    pub async fn insert_posts_handler(&self, posts: Vec<Post>) -> Result<(), sqlx::Error> {
        // 清除舊資料
        let _ = &self.delete_posts().await;

        let mut query_builder = QueryBuilder::new(
            r#"
            INSERT INTO hackmd_posts (
                id, content, created_at, last_changed_at, user_path, 
                permalink, publish_link, publish_type, published_at, 
                read_permission, short_id, tags, tags_updated_at, 
                team_path, title, title_updated_at, write_permission
            )
            "#,
        );

        query_builder.push_values(posts, |mut b, post| {
            b.push_bind(post.id)
                .push_bind(post.content)
                .push_bind(post.created_at)
                .push_bind(post.last_changed_at)
                .push_bind(post.user_path)
                .push_bind(post.permalink)
                .push_bind(post.publish_link)
                .push_bind(post.publish_type)
                .push_bind(post.published_at)
                .push_bind(post.read_permission)
                .push_bind(post.short_id)
                .push_bind(post.tags)
                .push_bind(post.tags_updated_at)
                .push_bind(post.team_path)
                .push_bind(post.title)
                .push_bind(post.title_updated_at)
                .push_bind(post.write_permission);
        });

        let query = query_builder.build();

        query.execute(&self.get_pool()).await?;

        Ok(())
    }

    /// 取得所有 blogs
    pub async fn get_all_blogs(&self) -> Result<Vec<DbBlog>, sqlx::Error> {
        let blogs = sqlx::query_as::<_, DbBlog>(
            r#"
            SELECT id, markdown, html, tags, created_at, updated_at
            FROM blogs
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.get_pool())
        .await?;

        Ok(blogs)
    }

    /// 取得特定 blog
    pub async fn get_blog_by_id(&self, id: uuid::Uuid) -> Result<DbBlog, sqlx::Error> {
        let blog = sqlx::query_as::<_, DbBlog>(
            r#"
            SELECT id, markdown, html, tags, created_at, updated_at
            FROM blogs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.get_pool())
        .await?;

        Ok(blog)
    }

    /// 刪除特定 blog
    pub async fn delete_blog(&self, id: uuid::Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            DELETE FROM blogs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.get_pool())
        .await?;

        Ok(())
    }

    /// insert or update blog
    pub async fn upsert_blog(
        &self,
        id: uuid::Uuid,
        markdown: String,
        html: String,
        tags: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        let query = r#"
            INSERT INTO blogs (id, markdown, html, tags, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (id)
            DO UPDATE SET
                markdown = EXCLUDED.markdown,
                html = EXCLUDED.html,
                tags = EXCLUDED.tags,
                updated_at = NOW();
        "#;

        sqlx::query(query)
            .bind(id) // $1
            .bind(markdown) // $2
            .bind(html) // $3
            .bind(tags) // $4
            .execute(&self.get_pool())
            .await?;

        Ok(())
    }
}
