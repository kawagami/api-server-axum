use axum::extract::ws::{Message, WebSocket};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use futures::stream::SplitSink;
use reqwest::Client;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::{broadcast, Mutex};

use crate::storage::Storage;
use crate::structs::config::AppConfig;
use crate::structs::ws::WsEvent;

pub struct AppStateInner {
    pub pg_pool: Pool<Postgres>,
    pub redis_pool: RedisPool<RedisConnectionManager>,
    pub http_client: Client,
    pub tx: broadcast::Sender<String>,
    pub connections: ConnectionMap,
    pub storage: Storage,
    pub config: AppConfig,
    pub settings: Arc<RwLock<HashMap<String, String>>>,
}

impl AppStateInner {
    pub async fn new() -> Self {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");

        let pg_pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        let redis_host = std::env::var("REDIS_HOST").expect("找不到 REDIS_HOST");
        let manager = RedisConnectionManager::new(format!("redis://{}:6379", redis_host))
            .expect("Failed to create Redis connection manager");
        let redis_pool = bb8::Pool::builder()
            .build(manager)
            .await
            .expect("Failed to build Redis connection pool");

        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        let (tx, _rx) = broadcast::channel(100);

        Self {
            pg_pool,
            redis_pool,
            http_client,
            tx,
            connections: Arc::new(Mutex::new(HashMap::new())),
            storage: Storage::from_env(),
            config: AppConfig::from_env(),
            settings: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub type ConnectionMap = Arc<Mutex<HashMap<SocketAddr, TrackedConnection>>>;

pub struct TrackedConnection {
    pub connected_at: std::time::SystemTime,
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub user_email: Option<String>,
    pub real_ip: String,
}

#[derive(Serialize)]
pub struct DisplayTrackedConnection {
    pub addr: String,
    pub connected_at: std::time::SystemTime,
    pub user_email: Option<String>,
    pub real_ip: String,
}

#[derive(Clone)]
pub struct Settings(Arc<RwLock<HashMap<String, String>>>);

impl Settings {
    pub fn get(&self, key: &str) -> Option<String> {
        self.0.read().unwrap().get(key).cloned()
    }

    pub async fn reload(&self, pool: &Pool<Postgres>) {
        match crate::repositories::app_settings::get_all(pool).await {
            Ok(rows) => {
                *self.0.write().unwrap() = rows.into_iter().map(|s| (s.key, s.value)).collect();
            }
            Err(e) => {
                tracing::error!("Failed to reload app_settings: {:?}", e);
            }
        }
    }
}

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub async fn new() -> Self {
        let app_state = AppStateInner::new().await;
        AppState(Arc::new(app_state))
    }

    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.0.pg_pool
    }

    pub async fn get_redis_conn(
        &self,
    ) -> Result<bb8::PooledConnection<'_, RedisConnectionManager>, redis::RedisError> {
        self.0.redis_pool.get().await.map_err(|e| {
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

    pub fn get_redis_pool(&self) -> &RedisPool<RedisConnectionManager> {
        &self.0.redis_pool
    }

    pub fn get_http_client(&self) -> &Client {
        &self.0.http_client
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

    pub fn get_config(&self) -> &AppConfig {
        &self.0.config
    }

    pub fn get_settings(&self) -> Settings {
        Settings(self.0.settings.clone())
    }

    pub async fn reload_settings(&self) {
        match crate::repositories::app_settings::get_all(self.get_pool()).await {
            Ok(rows) => {
                *self.0.settings.write().unwrap() =
                    rows.into_iter().map(|s| (s.key, s.value)).collect();
            }
            Err(e) => {
                tracing::error!("Failed to reload app_settings: {:?}", e);
            }
        }
    }

    pub fn broadcast(&self, event: WsEvent, data: serde_json::Value) {
        let msg = serde_json::json!({
            "type": event.as_str(),
            "data": data
        })
        .to_string();
        let _ = self.0.tx.send(msg);
    }
}

