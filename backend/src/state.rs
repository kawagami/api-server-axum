use axum::extract::ws::{Message, WebSocket};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use futures::{stream::SplitSink, SinkExt};
use reqwest::Client;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::sync::Mutex;

use crate::games::registry::GameRegistry;
use crate::services::torrents::TorrentManager;
use crate::storage::Storage;
use crate::structs::config::AppConfig;
use crate::structs::features::Feature;
use crate::structs::ws::WsEvent;
use std::collections::HashSet;

pub struct AppStateInner {
    pub pg_pool: Pool<Postgres>,
    pub redis_pool: RedisPool<RedisConnectionManager>,
    pub http_client: Client,
    pub connections: ConnectionMap,
    pub storage: Storage,
    pub config: AppConfig,
    pub settings: Arc<RwLock<HashMap<String, String>>>,
    /// enabled_features 設定的 parse 結果（reload 時更新）；None = 全開
    pub enabled_features: Arc<RwLock<Option<HashSet<Feature>>>>,
    pub torrents: TorrentManager,
    pub games: GameRegistry,
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

        Self {
            pg_pool,
            redis_pool,
            http_client,
            connections: Arc::new(Mutex::new(HashMap::new())),
            storage: Storage::from_env(),
            config: AppConfig::from_env(),
            settings: Arc::new(RwLock::new(HashMap::new())),
            enabled_features: Arc::new(RwLock::new(None)),
            torrents: TorrentManager::new().await,
            games: GameRegistry::new(),
        }
    }
}

pub type ConnectionMap = Arc<Mutex<HashMap<SocketAddr, TrackedConnection>>>;
pub type WsSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub struct TrackedConnection {
    pub connected_at: std::time::SystemTime,
    pub sender: WsSender,
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
pub struct Settings {
    map: Arc<RwLock<HashMap<String, String>>>,
    /// None = 全開；reload 時由 enabled_features 設定值 parse 而來，檢查是 sync set lookup
    enabled_features: Arc<RwLock<Option<HashSet<Feature>>>>,
}

impl Settings {
    pub fn new(
        map: Arc<RwLock<HashMap<String, String>>>,
        enabled_features: Arc<RwLock<Option<HashSet<Feature>>>>,
    ) -> Self {
        Self { map, enabled_features }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.map.read().unwrap().get(key).cloned()
    }

    pub fn feature_enabled(&self, feature: Feature) -> bool {
        match self.enabled_features.read().unwrap().as_ref() {
            None => true,
            Some(set) => set.contains(&feature),
        }
    }

    pub async fn reload(&self, pool: &Pool<Postgres>) {
        match crate::repositories::app_settings::get_all(pool).await {
            Ok(rows) => {
                let map: HashMap<String, String> =
                    rows.into_iter().map(|s| (s.key, s.value)).collect();
                let enabled = map
                    .get("enabled_features")
                    .and_then(|v| Feature::parse_setting(v));
                *self.map.write().unwrap() = map;
                *self.enabled_features.write().unwrap() = enabled;
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

    pub fn get_connections(&self) -> &ConnectionMap {
        &self.0.connections
    }

    pub fn get_storage(&self) -> &Storage {
        &self.0.storage
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.0.config
    }

    pub fn get_torrents(&self) -> &TorrentManager {
        &self.0.torrents
    }

    pub fn games(&self) -> &GameRegistry {
        &self.0.games
    }

    /// 點對點送文字訊息給單一連線（找不到連線就靜默丟棄）。
    pub fn send_to(&self, addr: SocketAddr, msg: String) {
        let connections = self.0.connections.clone();
        tokio::spawn(async move {
            let sender = {
                let conns = connections.lock().await;
                conns.get(&addr).map(|c| c.sender.clone())
            };
            if let Some(sender) = sender {
                let mut guard = sender.lock().await;
                if let Err(e) = guard.send(Message::Text(msg.into())).await {
                    tracing::warn!("send_to {} failed: {}", addr, e);
                }
            }
        });
    }

    pub fn get_settings(&self) -> Settings {
        Settings::new(self.0.settings.clone(), self.0.enabled_features.clone())
    }

    pub async fn reload_settings(&self) {
        self.get_settings().reload(self.get_pool()).await;
    }

    pub fn broadcast(&self, event: WsEvent, data: serde_json::Value) {
        self.broadcast_filtered(crate::structs::ws::envelope(event.as_str(), data), false);
    }

    /// 只推給已通過 admin 驗證的連線（user_email 有值）— 含 IP/email 等個資的事件走這裡
    pub fn broadcast_to_admins(&self, event: WsEvent, data: serde_json::Value) {
        self.broadcast_filtered(crate::structs::ws::envelope(event.as_str(), data), true);
    }

    /// 廣播 — 先複製 sender 清單釋放 map lock，
    /// 再 per-connection spawn，慢速客戶端不會卡住其他連線
    fn broadcast_filtered(&self, msg: String, admins_only: bool) {
        let connections = self.0.connections.clone();
        tokio::spawn(async move {
            let senders: Vec<(SocketAddr, WsSender)> = {
                let conns = connections.lock().await;
                conns
                    .iter()
                    .filter(|(_, c)| !admins_only || c.user_email.is_some())
                    .map(|(addr, c)| (*addr, c.sender.clone()))
                    .collect()
            };
            for (addr, sender) in senders {
                let msg = msg.clone();
                tokio::spawn(async move {
                    let mut guard = sender.lock().await;
                    if let Err(e) = guard.send(Message::Text(msg.into())).await {
                        tracing::warn!("broadcast to {} failed: {}", addr, e);
                    }
                });
            }
        });
    }
}

