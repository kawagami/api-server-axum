use crate::{repositories::users, state::AppStateV2, structs::users::User};
use axum::{extract::State, routing::get, Json, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", get(get_users))
}

/// 取 users 清單
async fn get_users(State(state): State<AppStateV2>) -> Json<Vec<User>> {
    let result = users::get_users(&state).await;

    match result {
        Ok(users) => Json(users),
        Err(err) => {
            tracing::error!("{}", err);
            Json(vec![])
        }
    }
}
