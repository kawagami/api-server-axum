use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new().route("/test", get(test))
}

pub async fn test() -> &'static str {
    "this is v1 test page"
}
