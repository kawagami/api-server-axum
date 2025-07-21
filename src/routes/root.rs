use crate::{errors::AppError, state::AppStateV2};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", get(index))
}

pub async fn index(State(_state): State<AppStateV2>) -> Result<Json<&'static str>, AppError> {
    Ok(Json("api server index page"))
}

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "empty page")
}
