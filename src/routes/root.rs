use crate::{errors::AppError, state::AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};

pub fn new() -> Router<AppState> {
    Router::new().route("/", get(index))
}

pub async fn index(State(_state): State<AppState>) -> Result<Json<&'static str>, AppError> {
    Ok(Json("api server index page"))
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "empty page")
}
