use crate::{
    errors::AppError,
    games::registry::GameSummary,
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, State},
    routing::get,
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(games_overview)))
}

/// 即時對局總覽：每款遊戲的等待 / 進行中桌數、在玩人數、排隊與大廳訂閱數。
/// 記憶體狀態，重啟即歸零；匿名對戰，不含玩家身份。
async fn games_overview(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<GameSummary>>, AppError> {
    auth_user.require_permission(Perm::GameRead)?;
    Ok(Json(state.games().summaries().await))
}
