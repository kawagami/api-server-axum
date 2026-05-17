use crate::{
    errors::AppError,
    middleware::auth,
    services::auth as auth_service,
    state::AppState,
    structs::auth::{AuthenticatedUser, ChangePasswordData, SignInData},
};
use axum::{
    extract::{Extension, Json, State},
    middleware,
    routing::{get, post},
    Router,
};
use serde::Serialize;

pub fn new(state: AppState) -> Router<AppState> {
    let me_route = Router::new()
        .route("/me", get(me))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ));

    let refresh_route = Router::new()
        .route("/refresh", post(refresh))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ));

    let change_password_route = Router::new()
        .route("/change_password", post(change_password))
        .layer(middleware::from_fn_with_state(
            state,
            auth::authorize_and_load,
        ));

    Router::new()
        .route("/", post(sign_in))
        .merge(me_route)
        .merge(refresh_route)
        .merge(change_password_route)
}

#[derive(Serialize)]
struct MeResponse {
    email: String,
    permissions: Vec<String>,
}

async fn sign_in(
    State(state): State<AppState>,
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

async fn refresh(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<String>, AppError> {
    let token = auth_service::refresh_admin_token(&state, auth_user.email).await?;
    Ok(Json(token))
}

async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<ChangePasswordData>,
) -> Result<(), AppError> {
    auth_service::change_password(&state, &auth_user.email, &body.current_password, &body.new_password).await
}
