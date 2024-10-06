use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{collections::HashSet, time::Duration};
use tokio::sync::broadcast;

use crate::structs::ws::FixedMessageContainer;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub tx: broadcast::Sender<String>,
    pub user_set: HashSet<String>,
    pub fixed_message_container: FixedMessageContainer,
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

        let fixed_message_container = FixedMessageContainer::new(10);

        Self {
            pool,
            tx,
            user_set,
            fixed_message_container,
        }
    }
}
