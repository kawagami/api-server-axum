use crate::{
    errors::AppError,
    middleware::auth,
    services::auth as auth_service,
    state::AppStateV2,
    structs::auth::{AuthenticatedUser, SignInData},
};
use axum::{
    extract::{Extension, Json, State},
    middleware,
    routing::{get, post},
    Router,
};
use serde::Serialize;

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    let me_route = Router::new()
        .route("/me", get(me))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ));

    Router::new()
        .route("/", post(sign_in))
        .merge(me_route)
}

#[derive(Serialize)]
struct MeResponse {
    email: String,
    permissions: Vec<String>,
}

async fn sign_in(
    State(state): State<AppStateV2>,
    Json(user_data): Json<SignInData>,
) -> Result<Json<String>, AppError> {
    let token = auth_service::sign_in(&state, &user_data.email, &user_data.password).await?;
    Ok(Json(token))
}

async fn me(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Json<MeResponse> {
    Json(MeResponse {
        email: auth_user.email,
        permissions: auth_user.permissions,
    })
}
