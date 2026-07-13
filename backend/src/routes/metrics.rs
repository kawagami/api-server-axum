use crate::{
    errors::AppError,
    repositories::system_metrics::{self as metrics_repo, SystemMetric},
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct MetricsQuery {
    /// 取近 N 小時,預設 24,上限 168(7 天)。
    hours: Option<i64>,
}

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(state, Router::new().route("/", get(get_metrics_handler)))
}

async fn get_metrics_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<Vec<SystemMetric>>, AppError> {
    auth_user.require_permission(Perm::MetricRead)?;
    let hours = query.hours.unwrap_or(24).clamp(1, 168);
    let metrics = metrics_repo::get_recent(state.get_pool(), hours).await?;
    Ok(Json(metrics))
}
