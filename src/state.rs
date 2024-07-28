use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
    time::Duration,
};

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::sync::broadcast;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub some_data: HashMap<String, String>,
    pub tx: broadcast::Sender<String>,
    pub user_set: HashSet<String>,
}

impl AppState {
    pub async fn new() -> Self {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");
        let (tx, _rx) = broadcast::channel(100);
        let user_set = HashSet::new();

        // set up connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&db_connection_str)
            .await
            .expect("can't connect to database");

        Self {
            pool,
            some_data: HashMap::default(),
            tx,
            user_set,
        }
    }
}
