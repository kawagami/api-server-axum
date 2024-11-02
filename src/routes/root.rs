use crate::state::AppStateV2;
use axum::{extract::State, http::StatusCode, response::IntoResponse};

// we can extract the connection pool with `State`

// #[debug_handler]
pub async fn using_connection_pool_extractor(
    State(state): State<AppStateV2>,
) -> Result<String, impl IntoResponse> {
    let pool = state.get_pool().await;

    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
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
