use axum::{routing::get, Router};

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/test", get(test))
}

pub async fn test() -> &'static str {
    "this is v2 test page"
}
