use crate::{
    errors::AppError,
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
    let protected = super::with_auth(
        state.clone(),
        Router::new()
            .route("/me", get(me))
            .route("/refresh", post(refresh))
            .route("/change_password", post(change_password)),
    );

    // 密碼登入端點掛 per-IP 限流，防爆破與 bcrypt CPU 耗盡
    Router::new()
        .route("/", post(sign_in))
        .layer(middleware::from_fn_with_state(
            state,
            crate::middleware::rate_limit::auth_rate_limit,
        ))
        .merge(protected)
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
    let token = auth_service::sign_in(
        state.get_pool(),
        state.get_redis_pool(),
        &state.get_config().jwt_secret,
        &user_data.email,
        &user_data.password,
    )
    .await?;
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
    let token = auth_service::refresh_admin_token(
        state.get_redis_pool(),
        &state.get_config().jwt_secret,
        auth_user.email,
    )
    .await?;
    Ok(Json(token))
}

async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<ChangePasswordData>,
) -> Result<(), AppError> {
    auth_service::change_password(
        state.get_pool(),
        &auth_user.email,
        &body.current_password,
        &body.new_password,
    )
    .await
}
