use crate::extract::{Json, Path};
use crate::{
    errors::AppError,
    services::app_settings as settings_service,
    state::AppState,
    structs::{
        app_settings::{AppSetting, UpdateSetting, UpdateSettings},
        auth::AuthenticatedUser,
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, State},
    response::IntoResponse,
    routing::{get, patch},
    Router
};
use std::collections::BTreeMap;

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(list_settings).patch(update_many))
            .route("/{key}", patch(update)),
    )
}

/// GET /settings/public — 無認證，訪客 SSR 用；只回白名單設定
pub fn public() -> Router<AppState> {
    Router::new().route("/public", get(public_settings))
}

/// 全站訪客拿到的是同一份白名單設定，沒有任何 per-user 差異，故可掛公開快取標頭。
/// 後台改主題後最久 60 秒才會在這一層更新（前端另有 `api/settings.ts` 的 revalidate 60）。
async fn public_settings(State(state): State<AppState>) -> impl IntoResponse {
    (
        super::public_cache(),
        Json(settings_service::get_public(&state.get_settings())),
    )
}

async fn list_settings(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<BTreeMap<String, Vec<AppSetting>>>, AppError> {
    auth_user.require_permission(Perm::SettingRead)?;
    let include_reserved = auth_user.has_permission(Perm::PlatformRead);
    Ok(Json(settings_service::get_all(state.get_pool(), include_reserved).await?))
}

/// 平台保留 key（如 enabled_features）走 platform:update，一般設定走 setting:update
fn require_update_permission(auth_user: &AuthenticatedUser, key: &str) -> Result<(), AppError> {
    if settings_service::is_reserved(key) {
        auth_user.require_permission(Perm::PlatformUpdate)
    } else {
        auth_user.require_permission(Perm::SettingUpdate)
    }
}

async fn update(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSetting>,
) -> Result<Json<AppSetting>, AppError> {
    require_update_permission(&auth_user, &key)?;
    let settings = state.get_settings();
    Ok(Json(settings_service::update(state.get_pool(), &settings, &key, &payload.value).await?))
}

/// 批次更新多個 key（同 transaction、全過才寫）。
/// 給「互相約束的設定組」用 —— 逐 key PATCH 換不過去的組合走這支，
/// 例如 webauthn_rp_id / webauthn_rp_origin 整組換網域。
async fn update_many(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateSettings>,
) -> Result<Json<Vec<AppSetting>>, AppError> {
    for key in payload.values.keys() {
        require_update_permission(&auth_user, key)?;
    }
    let settings = state.get_settings();
    Ok(Json(
        settings_service::update_many(state.get_pool(), &settings, &payload.values).await?,
    ))
}
