use crate::extract::{Json, Query};
use crate::{
    errors::AppError,
    services::stats as stats_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        roles::Perm,
        stats::{VisitorsQuery, VisitorsStats},
    },
};
use axum::{
    extract::{Extension, State},
    routing::get,
    Router
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/visitors", get(visitors_stats)))
}

/// 網站每日不重複到訪統計：今日即時值 + 期間合併去重 + 歷史趨勢。
async fn visitors_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<VisitorsQuery>,
) -> Result<Json<VisitorsStats>, AppError> {
    auth_user.require_permission(Perm::StatRead)?;
    Ok(Json(stats_service::visitors_stats(&state, query.days).await?))
}
