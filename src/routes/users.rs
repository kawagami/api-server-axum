use crate::{
    errors::AppError,
    repositories::users,
    routes::auth,
    state::AppStateV2,
    structs::users::{NewUser, User},
};
use axum::{
    extract::State,
    middleware,
    routing::{get, post},
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    let protected_routes = Router::new()
        .route("/", post(create_user).delete(delete_user))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ));

    Router::new()
        .route("/", get(get_users))
        .merge(protected_routes)
}

async fn get_users(State(state): State<AppStateV2>) -> Result<Json<Vec<User>>, AppError> {
    let result = users::get_users(&state).await.map_err(AppError::from)?;

    Ok(Json(result))
}

async fn create_user(
    State(state): State<AppStateV2>,
    Json(user): Json<NewUser>,
) -> Result<Json<bool>, AppError> {
    let _result = users::create_user(&state, user)
        .await
        .map_err(AppError::from)?;

    Ok(Json(true))
}

async fn delete_user(
    State(_state): State<AppStateV2>,
    Json(_user): Json<User>,
) -> Result<Json<bool>, AppError> {
    // let result = users::get_users(&state).await.map_err(AppError::from)?;

    Ok(Json(true))
}
