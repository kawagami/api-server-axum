use crate::{
    errors::AppError,
    services::messages as messages_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser, messages::MessagePaginatedResponse, pagination::PageQuery,
        roles::Perm,
    },
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(list_messages))
            .route("/{id}", axum::routing::delete(delete_message)),
    )
}

/// 訪客留言分頁列表(新到舊)
async fn list_messages(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(page): Query<PageQuery>,
) -> Result<Json<MessagePaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::MessageRead)?;
    let (limit, offset) = page.to_limit_offset(50);
    Ok(Json(
        messages_service::list(state.get_pool(), limit, offset).await?,
    ))
}

/// 刪除一則留言
async fn delete_message(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::MessageDelete)?;
    messages_service::delete(state.get_pool(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
