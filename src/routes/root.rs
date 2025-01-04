use crate::{errors::internal_error, state::AppStateV2};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", get(using_connection_pool_extractor))
}

pub async fn using_connection_pool_extractor(
    State(state): State<AppStateV2>,
) -> Result<String, impl IntoResponse> {
    let pool = state.get_pool();

    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(&pool)
        .await
        .map_err(internal_error)
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
