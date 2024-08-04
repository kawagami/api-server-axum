use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;
use tokio::sync::Mutex;

// we can extract the connection pool with `State`

// #[debug_handler]
pub async fn using_connection_pool_extractor(
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<String, impl IntoResponse> {
    let pool = &state.lock().await.pool;

    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(pool)
        .await
        .map_err(internal_error)
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
