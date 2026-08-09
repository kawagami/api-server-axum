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
use tokio::sync::{mpsc, Mutex};

use crate::games::registry::GameRegistry;
use crate::repositories::audit_logs::AuditEntry;
use crate::batch_writer::CHANNEL_CAPACITY as AUDIT_CHANNEL_CAPACITY;
use crate::services::system_metrics::CpuTimes;
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
    /// 稽核紀錄的批次寫入佇列（消費端 = `services::audit_logs::audit_writer`）
    pub audit_tx: mpsc::Sender<AuditEntry>,
    /// 由 app_settings 的 webauthn_rp_id / webauthn_rp_origin 建構（reload 時重建）；
    /// None = 設定缺漏或無效，passkey 端點回錯、密碼登入不受影響
    pub webauthn: Arc<RwLock<Option<webauthn_rs::Webauthn>>>,
    /// 上一次讀到的 /proc/stat 累計值，`CollectSystemMetrics` 用來算整個採樣間隔的 CPU 平均。
    /// None = 行程剛起來還沒有基準（第一輪不落地）。
    pub cpu_times: Arc<RwLock<Option<CpuTimes>>>,
}

impl AppStateInner {
    /// 一併回傳稽核佇列的接收端 —— 呼叫端必須把它交給
    /// `services::audit_logs::audit_writer`，否則佇列會塞滿、稽核靜默消失。
    pub async fn new() -> (Self, mpsc::Receiver<AuditEntry>) {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");

        let pg_pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        let redis_url = crate::repositories::redis::redis_url_from_env();
        let manager = RedisConnectionManager::new(redis_url)
            .expect("Failed to create Redis connection manager");
        let redis_pool = bb8::Pool::builder()
            .build(manager)
            .await
            .expect("Failed to build Redis connection pool");

        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        let (audit_tx, audit_rx) = mpsc::channel(AUDIT_CHANNEL_CAPACITY);

        let inner = Self {
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
            audit_tx,
            webauthn: Arc::new(RwLock::new(None)),
            cpu_times: Arc::new(RwLock::new(None)),
        };
        (inner, audit_rx)
    }
}

pub type ConnectionMap = Arc<Mutex<HashMap<SocketAddr, TrackedConnection>>>;
pub type WsSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

pub struct TrackedConnection {
    pub connected_at: std::time::SystemTime,
    pub sender: WsSender,
    pub user_email: Option<String>,
    pub real_ip: String,
    pub user_agent: String,
}

#[derive(Serialize)]
pub struct DisplayTrackedConnection {
    pub addr: String,
    /// ISO-8601 毫秒 UTC 字串（`SystemTime` 預設序列化成 `{secs,nanos}` 物件，前端不好用）。
    /// 固定寬度，字典序 == 時間序，可直接拿來排序。
    pub connected_at: String,
    pub user_email: Option<String>,
    pub real_ip: String,
    pub user_agent: String,
}

#[derive(Clone)]
pub struct Settings {
    map: Arc<RwLock<HashMap<String, String>>>,
    /// None = 全開；reload 時由 enabled_features 設定值 parse 而來，檢查是 sync set lookup
    enabled_features: Arc<RwLock<Option<HashSet<Feature>>>>,
    /// reload 時由 webauthn_rp_id / webauthn_rp_origin 建構；None = 未設定/無效
    webauthn: Arc<RwLock<Option<webauthn_rs::Webauthn>>>,
}

/// 從 settings map 建 Webauthn instance；缺值/無效回 None（只記 log，不擋其他設定重載）
fn build_webauthn(map: &HashMap<String, String>) -> Option<webauthn_rs::Webauthn> {
    let rp_id = map.get("webauthn_rp_id").filter(|v| !v.is_empty())?;
    let rp_origin = map.get("webauthn_rp_origin").filter(|v| !v.is_empty())?;

    let url = match webauthn_rs::prelude::Url::parse(rp_origin) {
        Ok(url) => url,
        Err(e) => {
            tracing::error!("webauthn_rp_origin 不是合法 URL: {:?}", e);
            return None;
        }
    };
    match webauthn_rs::WebauthnBuilder::new(rp_id, &url)
        .and_then(|b| b.rp_name("Kawa Admin").build())
    {
        Ok(webauthn) => Some(webauthn),
        Err(e) => {
            tracing::error!("Webauthn 建構失敗（檢查 webauthn_rp_id / webauthn_rp_origin）: {:?}", e);
            None
        }
    }
}

impl Settings {
    pub fn new(
        map: Arc<RwLock<HashMap<String, String>>>,
        enabled_features: Arc<RwLock<Option<HashSet<Feature>>>>,
        webauthn: Arc<RwLock<Option<webauthn_rs::Webauthn>>>,
    ) -> Self {
        Self { map, enabled_features, webauthn }
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

    /// Webauthn instance（Clone 出去用）；None = 設定缺漏或無效
    pub fn webauthn(&self) -> Option<webauthn_rs::Webauthn> {
        self.webauthn.read().unwrap().clone()
    }

    pub async fn reload(&self, pool: &Pool<Postgres>) {
        match crate::repositories::app_settings::get_all(pool).await {
            Ok(rows) => {
                let map: HashMap<String, String> =
                    rows.into_iter().map(|s| (s.key, s.value)).collect();
                let enabled = map
                    .get("enabled_features")
                    .and_then(|v| Feature::parse_setting(v));
                let webauthn = build_webauthn(&map);
                // logs 表的落地門檻。值住在 logging.rs 的 static（subscriber 是 process
                // 全域、且早於 AppState 就存在），這裡只負責把設定推過去。
                if let Some(level) = map.get("log_db_level") {
                    crate::logging::set_db_level(level);
                }
                *self.map.write().unwrap() = map;
                *self.enabled_features.write().unwrap() = enabled;
                *self.webauthn.write().unwrap() = webauthn;
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
    /// 見 [`AppStateInner::new`]：回傳的 receiver 必須交給稽核批次寫入器。
    pub async fn new() -> (Self, mpsc::Receiver<AuditEntry>) {
        let (app_state, audit_rx) = AppStateInner::new().await;
        (AppState(Arc::new(app_state)), audit_rx)
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
                    redis::ErrorKind::Io,
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

    /// 稽核佇列的送出端（audit middleware 用）
    pub fn get_audit_tx(&self) -> &mpsc::Sender<AuditEntry> {
        &self.0.audit_tx
    }

    /// CPU 採樣基準：上一輪讀到的 /proc/stat 累計值（行程剛起來時為 None）。
    /// 讀寫都只有 `CollectSystemMetrics` 一個呼叫端，而 scheduler 的 per-job guard 保證
    /// 上一輪沒跑完就跳過本輪，所以 get 與 set 分兩次 lock 不會交錯。
    pub fn cpu_times(&self) -> Option<CpuTimes> {
        *self.0.cpu_times.read().unwrap()
    }

    pub fn set_cpu_times(&self, now: CpuTimes) {
        *self.0.cpu_times.write().unwrap() = Some(now);
    }

    /// 點對點送文字訊息給單一連線（找不到連線就靜默丟棄）。
    pub fn send_to(&self, addr: SocketAddr, msg: String) {
        self.send_many(addr, vec![msg]);
    }

    /// 依序送多則訊息給同一連線 —— **抵達順序保證與 `msgs` 的順序一致**。
    ///
    /// 為什麼需要這個而不是連呼 `send_to`：每次 spawn 出去的 task 各自去搶該連線的
    /// sender lock，取得順序不保證等於 spawn 順序。遊戲協定依賴順序（阿瓦隆的私有
    /// `role_assigned` 必須早於公開的階段訊息 —— 前端的 `your_seat` 只從前者取；
    /// 2 人局的 `move_made` 必須早於 `game_over`），倒序抵達會讓客戶端在還不知道
    /// 自己是誰的狀態下處理階段更新。
    ///
    /// 做法：整批只 spawn 一個 task，sender lock **一次取得、送完才放**，
    /// 順帶擋掉別的 send 插進批次中間。
    pub fn send_many(&self, addr: SocketAddr, msgs: Vec<String>) {
        if msgs.is_empty() {
            return;
        }
        let connections = self.0.connections.clone();
        tokio::spawn(async move {
            let sender = {
                let conns = connections.lock().await;
                conns.get(&addr).map(|c| c.sender.clone())
            };
            let Some(sender) = sender else { return };
            let mut guard = sender.lock().await;
            for msg in msgs {
                if let Err(e) = guard.send(Message::Text(msg.into())).await {
                    // 同 ws.rs 的收訊錯誤：對端關頁/睡眠時送出失敗是常態，清理照常走
                    tracing::debug!("send_to {} failed: {}", addr, e);
                    // 連線已壞，同批後續訊息沒有意義；清理交給 handle_socket
                    break;
                }
            }
        });
    }

    /// 送出一整批 outbox（遊戲框架的統一出口）。
    ///
    /// 同一收件人的多則訊息會併成一次有序送出（見 [`AppState::send_many`]），
    /// 不同收件人之間平行 —— 慢速客戶端不會拖住其他人。
    pub fn send_outbox(&self, outbox: Vec<(SocketAddr, String)>) {
        for (addr, msgs) in group_by_addr(outbox) {
            self.send_many(addr, msgs);
        }
    }
    pub fn get_settings(&self) -> Settings {
        Settings::new(
            self.0.settings.clone(),
            self.0.enabled_features.clone(),
            self.0.webauthn.clone(),
        )
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
                        tracing::debug!("broadcast to {} failed: {}", addr, e);
                    }
                });
            }
        });
    }
}

/// 把 outbox 依收件人分組：**每組內維持原順序**，組間維持首次出現的順序。
///
/// 純函式、與 WS 無關，故可單測 —— 順序正確性就是它的全部職責。
/// 一批 outbox 最多幾十則，線性搜尋比 HashMap 省事，也不必再排序。
fn group_by_addr(outbox: Vec<(SocketAddr, String)>) -> Vec<(SocketAddr, Vec<String>)> {
    let mut grouped: Vec<(SocketAddr, Vec<String>)> = Vec::new();
    for (addr, msg) in outbox {
        match grouped.iter_mut().find(|(a, _)| *a == addr) {
            Some((_, msgs)) => msgs.push(msg),
            None => grouped.push((addr, vec![msg])),
        }
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(last: u8) -> SocketAddr {
        format!("127.0.0.1:{}", 10000 + last as u16).parse().unwrap()
    }

    fn out(pairs: &[(u8, &str)]) -> Vec<(SocketAddr, String)> {
        pairs.iter().map(|(a, m)| (addr(*a), m.to_string())).collect()
    }

    /// 同一收件人的訊息必須維持 push 順序 —— 這正是遊戲協定依賴的性質
    /// （私有 `role_assigned` 早於公開階段訊息、`move_made` 早於 `game_over`）。
    #[test]
    fn keeps_order_within_recipient() {
        let grouped = group_by_addr(out(&[
            (1, "role_assigned"),
            (2, "role_assigned"),
            (1, "phase"),
            (2, "phase"),
            (1, "lobby_update"),
        ]));

        assert_eq!(grouped.len(), 2, "兩個收件人應分成兩組");
        assert_eq!(grouped[0].0, addr(1));
        assert_eq!(grouped[0].1, ["role_assigned", "phase", "lobby_update"]);
        assert_eq!(grouped[1].0, addr(2));
        assert_eq!(grouped[1].1, ["role_assigned", "phase"]);
    }

    /// 組的順序＝首次出現順序（決定誰先被 spawn，行為可預期）
    #[test]
    fn keeps_first_appearance_order_between_recipients() {
        let grouped = group_by_addr(out(&[(3, "a"), (1, "b"), (3, "c"), (2, "d")]));
        let addrs: Vec<SocketAddr> = grouped.iter().map(|(a, _)| *a).collect();
        assert_eq!(addrs, [addr(3), addr(1), addr(2)]);
    }

    #[test]
    fn empty_outbox_groups_to_nothing() {
        assert!(group_by_addr(vec![]).is_empty());
    }
}

