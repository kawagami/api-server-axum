use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    pub connection: Pool<Postgres>,
}
