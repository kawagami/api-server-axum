use axum::extract::ws::{Message, WebSocket};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use futures::stream::SplitSink;
use reqwest::Client;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};

use crate::storage::Storage;
use crate::structs::ws::WsEvent;

pub struct AppStateInner {
    pub pool: Pool<Postgres>,
    pub redis_pool: RedisPool<RedisConnectionManager>,
    pub http_client: Client,
    pub fastapi_upload_host: String,
    pub tx: broadcast::Sender<String>,
    pub connections: ConnectionMap,
    pub storage: Storage,
}

impl AppStateInner {
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
            storage: Storage::from_env(),
        }
    }
}

pub type ConnectionMap = Arc<Mutex<HashMap<SocketAddr, TrackedConnection>>>;

pub struct TrackedConnection {
    pub connected_at: std::time::SystemTime,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub user_email: Option<String>,
}

#[derive(Serialize)]
pub struct DisplayTrackedConnection {
    pub addr: String,
    pub connected_at: std::time::SystemTime,
    pub user_email: Option<String>,
}

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub async fn new() -> Self {
        let app_state = AppStateInner::new().await;
        AppState(Arc::new(app_state))
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
            match e {
                bb8::RunError::User(redis_err) => redis_err,
                bb8::RunError::TimedOut => redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "Redis connection pool timed out",
                )),
            }
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

    pub fn get_storage(&self) -> &Storage {
        &self.0.storage
    }

    pub fn broadcast(&self, event: WsEvent, data: serde_json::Value) {
        let msg = serde_json::json!({
            "type": event.as_str(),
            "data": data
        })
        .to_string();
        let _ = self.get_tx().send(msg);
    }

    pub async fn broadcast_to_admins(&self, event: WsEvent, data: serde_json::Value) {
        use futures_util::SinkExt;
        let msg = serde_json::json!({
            "type": event.as_str(),
            "data": data
        })
        .to_string();
        let connections = self.get_connections().lock().await;
        for conn in connections.values() {
            if conn.user_email.is_some() {
                let mut sender = conn.sender.lock().await;
                let _ = sender.send(Message::Text(msg.clone().into())).await;
            }
        }
    }
}
