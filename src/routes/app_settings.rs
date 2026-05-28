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

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(get_all))
            .route("/:key", patch(update)),
    )
}

async fn get_all(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<AppSetting>>, AppError> {
    auth_user.require_permission(Perm::SettingRead)?;
    Ok(Json(settings_service::get_all(&state).await?))
}

async fn update(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSetting>,
) -> Result<Json<AppSetting>, AppError> {
    auth_user.require_permission(Perm::SettingUpdate)?;
    Ok(Json(settings_service::update(&state, &key, &payload.value).await?))
}
