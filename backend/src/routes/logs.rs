use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    repositories::logs::Log,
    services::logs as logs_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        logs::LogQuery,
        pagination::{PageQuery, Paginated},
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, State},
    routing::get,
    Router
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(list_logs))
            .route("/request/{request_id}", get(request_trace)),
    )
}

async fn list_logs(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(filter): Query<LogQuery>,
    Query(page): Query<PageQuery>,
) -> Result<Json<Paginated<Log>>, AppError> {
    auth_user.require_permission(Perm::LogRead)?;
    let (limit, offset) = page.to_limit_offset(100);
    Ok(Json(
        logs_service::get_logs(state.get_pool(), &filter, limit, offset).await?,
    ))
}

/// 一個請求的完整 log 軌跡（時間正序）。使用者回報時附的 `request_id`
/// 或錯誤回應 body / `x-request-id` header 的值直接丟進來就行。
///
/// 刻意**不分頁**（單一請求的 log 是個位數，上限見 `REQUEST_TRACE_LIMIT`），
/// 故回裸陣列而非 `Paginated`。
async fn request_trace(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(request_id): Path<String>,
) -> Result<Json<Vec<Log>>, AppError> {
    auth_user.require_permission(Perm::LogRead)?;
    let logs = logs_service::logs_by_request(state.get_pool(), &request_id).await?;
    Ok(Json(logs))
}
