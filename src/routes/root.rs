use crate::{
    errors::{AppError, SystemError},
    state::AppStateV2,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", get(health_check))
}

pub async fn health_check(State(state): State<AppStateV2>) -> Result<String, AppError> {
    sqlx::query_scalar("select 'hello world from pg'")
        .fetch_one(state.get_pool())
        .await
        .map_err(|err| {
            AppError::SystemError(SystemError::Internal(format!(
                "Database health check failed: {}",
                err
            )))
        })
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
