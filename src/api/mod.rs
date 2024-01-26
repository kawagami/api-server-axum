use axum::{http::StatusCode, response::IntoResponse, Router};

mod v1;
mod v2;

pub fn routes() -> Router {
    Router::new()
        .nest("/v1", v1::routes())
        .nest("/v2", v2::routes())
        .fallback(fallback)
}

async fn fallback() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "api not found")
}
