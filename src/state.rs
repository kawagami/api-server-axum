use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub some_data: HashMap<String, String>,
}

impl AppState {
    pub async fn new() -> Self {
        let db_connection_str = std::env::var("DATABASE_URL").expect("找不到 DATABASE_URL");

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
        }
    }
}
