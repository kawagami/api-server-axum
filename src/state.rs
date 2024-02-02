use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use sqlx::{Pool, Postgres};

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub pool: Pool<Postgres>,
    pub some_data: HashMap<String, String>,
}
