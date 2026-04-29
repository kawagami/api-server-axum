use crate::{
    errors::AppError,
    services::auth as auth_service,
    state::AppStateV2,
    structs::auth::SignInData,
};
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", post(sign_in))
}

async fn sign_in(
    State(state): State<AppStateV2>,
    Json(user_data): Json<SignInData>,
) -> Result<Json<String>, AppError> {
    let token = auth_service::sign_in(&state, &user_data.email, &user_data.password).await?;
    Ok(Json(token))
}
