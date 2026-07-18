use crate::{
    errors::AppError,
    services::app_settings as settings_service,
    state::AppState,
    structs::{
        app_settings::{AppSetting, UpdateSetting},
        auth::AuthenticatedUser,
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, Path, State},
    routing::{get, patch},
    Json, Router,
};
use std::collections::BTreeMap;

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(get_all))
            .route("/{key}", patch(update)),
    )
}

/// GET /settings/public — 無認證，訪客 SSR 用；只回白名單設定
pub fn public() -> Router<AppState> {
    Router::new().route("/public", get(get_public))
}

async fn get_public(State(state): State<AppState>) -> Json<BTreeMap<String, String>> {
    Json(settings_service::get_public(&state.get_settings()))
}

async fn get_all(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<BTreeMap<String, Vec<AppSetting>>>, AppError> {
    auth_user.require_permission(Perm::SettingRead)?;
    let include_reserved = auth_user.has_permission(Perm::PlatformRead);
    Ok(Json(settings_service::get_all(state.get_pool(), include_reserved).await?))
}

async fn update(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSetting>,
) -> Result<Json<AppSetting>, AppError> {
    // 平台保留 key（如 enabled_features）走 platform:update，一般設定走 setting:update
    if settings_service::is_reserved(&key) {
        auth_user.require_permission(Perm::PlatformUpdate)?;
    } else {
        auth_user.require_permission(Perm::SettingUpdate)?;
    }
    let settings = state.get_settings();
    Ok(Json(settings_service::update(state.get_pool(), &settings, &key, &payload.value).await?))
}
