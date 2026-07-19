use crate::{
    errors::AppError,
    services::{auth as auth_service, webauthn as webauthn_service},
    state::AppState,
    structs::{
        auth::{AuthenticatedUser, ChangePasswordData, SignInData},
        webauthn::{
            PasskeyListItem, PasskeyLoginBeginResponse, PasskeyLoginFinishData,
            PasskeyRegisterFinishData,
        },
    },
};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, post},
    Router,
};
use serde::Serialize;
use webauthn_rs::prelude::CreationChallengeResponse;

pub fn new(state: AppState) -> Router<AppState> {
    let protected = super::with_auth(
        state.clone(),
        Router::new()
            .route("/me", get(me))
            .route("/refresh", post(refresh))
            .route("/change_password", post(change_password))
            .route("/passkeys", get(list_passkeys))
            .route("/passkeys/{id}", delete(delete_passkey))
            .route("/passkeys/register/begin", post(passkey_register_begin))
            .route("/passkeys/register/finish", post(passkey_register_finish)),
    );

    // passkey 登入（公開）：獨立限流 scope，Conditional UI 每次載入登入頁就打一發 begin
    let passkey_login = Router::new()
        .route("/passkeys/login/begin", post(passkey_login_begin))
        .route("/passkeys/login/finish", post(passkey_login_finish))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::rate_limit::webauthn_rate_limit,
        ));

    // 密碼登入端點掛 per-IP 限流，防爆破與 bcrypt CPU 耗盡
    Router::new()
        .route("/", post(sign_in))
        .layer(middleware::from_fn_with_state(
            state,
            crate::middleware::rate_limit::auth_rate_limit,
        ))
        .merge(passkey_login)
        .merge(protected)
}

#[derive(Serialize)]
struct MeResponse {
    id: i64,
    name: String,
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
        &user_data.name,
        &user_data.password,
    )
    .await?;
    Ok(Json(token))
}

async fn me(
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Json<MeResponse> {
    Json(MeResponse {
        id: auth_user.id,
        name: auth_user.name,
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
        auth_user.id,
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
        auth_user.id,
        &body.current_password,
        &body.new_password,
    )
    .await
}

async fn passkey_register_begin(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<CreationChallengeResponse>, AppError> {
    Ok(Json(webauthn_service::begin_registration(&state, auth_user.id).await?))
}

async fn passkey_register_finish(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Json(body): Json<PasskeyRegisterFinishData>,
) -> Result<StatusCode, AppError> {
    webauthn_service::finish_registration(&state, auth_user.id, &body.label, &body.credential)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn passkey_login_begin(
    State(state): State<AppState>,
) -> Result<Json<PasskeyLoginBeginResponse>, AppError> {
    Ok(Json(webauthn_service::begin_login(&state).await?))
}

// 回傳與 POST /admin/auth 同形（Json<String> token），前端代理可照抄
async fn passkey_login_finish(
    State(state): State<AppState>,
    Json(body): Json<PasskeyLoginFinishData>,
) -> Result<Json<String>, AppError> {
    let token = webauthn_service::finish_login(&state, &body.auth_id, &body.credential).await?;
    Ok(Json(token))
}

async fn list_passkeys(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> Result<Json<Vec<PasskeyListItem>>, AppError> {
    Ok(Json(
        crate::repositories::passkeys::list_by_user_id(state.get_pool(), auth_user.id).await?,
    ))
}

async fn delete_passkey(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthenticatedUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let deleted =
        crate::repositories::passkeys::delete_own(state.get_pool(), auth_user.id, id).await?;
    if !deleted {
        return Err(AppError::RequestError(crate::errors::RequestError::NotFound));
    }
    Ok(StatusCode::NO_CONTENT)
}
