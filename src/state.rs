use axum::extract::ws::{Message, WebSocket};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use futures::stream::SplitSink;
use reqwest::Client;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub redis_pool: RedisPool<RedisConnectionManager>,
    pub http_client: Client,
    pub fastapi_upload_host: String,
    pub tx: broadcast::Sender<String>,
    pub connections: ConnectionMap, // 追蹤連線
}

impl AppState {
    pub async fn new() -> Self {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");

        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        // migration
        sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
            .await
            .expect("Migrator new fail")
            .run(&pool)
            .await
            .expect("Migrator run fail");

        let redis_host = std::env::var("REDIS_HOST").expect("找不到 REDIS_HOST");
        let manager = RedisConnectionManager::new(format!("redis://{}:6379", redis_host)).unwrap();
        let redis_pool = bb8::Pool::builder().build(manager).await.unwrap();

        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        let fastapi_upload_host =
            std::env::var("FASTAPI_UPLOAD_HOST").expect("找不到 FASTAPI_UPLOAD_HOST");

        let (tx, _rx) = broadcast::channel(100);

        Self {
            pool,
            redis_pool,
            http_client,
            fastapi_upload_host,
            tx,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub type ConnectionMap = Arc<Mutex<HashMap<SocketAddr, TrackedConnection>>>;

pub struct TrackedConnection {
    pub addr: String,
    pub connected_at: std::time::SystemTime,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>, // 保留 sender
}

#[derive(Serialize)]
pub struct DisplayTrackedConnection {
    pub addr: String,
    pub connected_at: std::time::SystemTime,
}

#[derive(Clone)]
pub struct AppStateV2(Arc<AppState>);

impl AppStateV2 {
    pub async fn new() -> Self {
        let app_state = AppState::new().await;
        AppStateV2(Arc::new(app_state))
    }

    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.0.pool
    }

    pub fn get_redis_pool(&self) -> &RedisPool<RedisConnectionManager> {
        &self.0.redis_pool
    }

    pub async fn get_redis_conn(
        &self,
    ) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, redis::RedisError> {
        self.get_redis_pool().get().await.map_err(|e| {
            tracing::error!("Failed to get Redis connection: {:?}", e);
            redis::RedisError::from((redis::ErrorKind::IoError, "Failed to get Redis connection"))
        })
    }

    pub fn get_http_client(&self) -> &Client {
        &self.0.http_client
    }

    pub fn get_fastapi_upload_host(&self) -> &str {
        &self.0.fastapi_upload_host
    }

    pub fn get_tx(&self) -> &broadcast::Sender<String> {
        &self.0.tx
    }

    pub fn get_connections(&self) -> &ConnectionMap {
        &self.0.connections
    }
}
