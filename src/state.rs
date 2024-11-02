use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::sync::{broadcast, Mutex};

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub tx: broadcast::Sender<String>,
    pub user_set: HashSet<String>,
}

impl AppState {
    pub async fn new() -> Self {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");
        let (tx, _rx) = broadcast::channel(64);
        let user_set = HashSet::new();

        // set up connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        Self { pool, tx, user_set }
    }
}

#[derive(Clone)]
pub struct AppStateV2(Arc<Mutex<AppState>>);
impl AppStateV2 {
    pub async fn new() -> Self {
        let app_state = AppState::new().await;
        AppStateV2(Arc::new(Mutex::new(app_state)))
    }

    pub async fn get_pool(&self) -> Pool<Postgres> {
        // 鎖定 Mutex，取得 AppState 的不可變引用
        let app_state = self.0.lock().await;
        // 回傳複製的 pool
        app_state.pool.clone()
    }

    pub async fn get_tx(&self) -> broadcast::Sender<String> {
        let app_state = self.0.lock().await;
        app_state.tx.clone()
    }

    // 檢查用戶是否存在於 `user_set` 中
    pub async fn contains_user(&self, user: &str) -> bool {
        let app_state = self.0.lock().await;
        app_state.user_set.contains(user)
    }

    // 取得所有 `user_set` 中的用戶名稱
    pub async fn get_all_users(&self) -> Vec<String> {
        let app_state = self.0.lock().await;
        app_state.user_set.iter().cloned().collect()
    }

    // 新增用戶至 `user_set`
    pub async fn add_user(&self, user: String) {
        let mut app_state = self.0.lock().await;
        app_state.user_set.insert(user);
    }

    // 從 `user_set` 中刪除用戶
    pub async fn remove_user(&self, user: &str) {
        let mut app_state = self.0.lock().await;
        app_state.user_set.remove(user);
    }
}
