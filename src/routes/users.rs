use crate::{errors::AppError, repositories::users, state::AppStateV2, structs::users::User};
use axum::{extract::State, routing::get, Json, Router};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", get(get_users).post(create_user))
}

async fn get_users(State(state): State<AppStateV2>) -> Result<Json<Vec<User>>, AppError> {
    let result = users::get_users(&state).await.map_err(AppError::from)?;

    Ok(Json(result))
}

async fn create_user(
    State(_state): State<AppStateV2>,
    Json(_user): Json<User>,
) -> Result<Json<bool>, AppError> {
    // let result = users::get_users(&state).await.map_err(AppError::from)?;


    Ok(Json(true))
}
