use crate::repositories::stocks;
use crate::state::AppStateV2;
use crate::{errors::AppError, routes::auth};
use axum::{extract::State, middleware, routing::get, Json, Router};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/get_codes", get(get_codes))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

pub async fn get_codes(State(state): State<AppStateV2>) -> Result<Json<usize>, AppError> {
    let response = stocks::get_codes(&state).await?;

    let count = stocks::save_codes(&state, &response).await?;

    Ok(Json(count))
}
