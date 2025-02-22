use crate::{errors::AppError, repositories::users, state::AppStateV2, structs::users::User};
use axum::{extract::State, routing::get, Json, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", get(get_users))
}

/// 取 users 清單
async fn get_users(State(state): State<AppStateV2>) -> Result<Json<Vec<User>>, AppError> {
    let result = users::get_users(&state).await.map_err(AppError::from)?;

    Ok(Json(result))
}
