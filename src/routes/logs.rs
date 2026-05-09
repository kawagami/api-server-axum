use crate::{
    errors::AppError,
    middleware::auth,
    repositories::logs::{get_logs, Log},
    state::AppState,
    structs::{auth::AuthenticatedUser, roles::Perm},
};
use axum::{
    extract::{Extension, Query, State},
    middleware,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct LogQuery {
    level: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    100
}

pub fn new(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_logs_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_and_load,
        ))
}

async fn get_logs_handler(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> Result<Json<Vec<Log>>, AppError> {
    auth_user.require_permission(Perm::LogRead)?;
    let logs = get_logs(state.get_pool(), query.level, query.limit, query.offset).await?;
    Ok(Json(logs))
}
