use crate::{
    errors::AppError,
    services::blog_comments as comments_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser, blog_comments::BlogCommentPaginatedResponse,
        pagination::PageQuery, roles::Perm,
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
            .route("/", get(list_comments))
            .route("/{id}", axum::routing::delete(delete_comment)),
    )
}

/// 全站 blog 留言分頁列表(新到舊)
async fn list_comments(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(page): Query<PageQuery>,
) -> Result<Json<BlogCommentPaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::CommentRead)?;
    let (limit, offset) = page.to_limit_offset(50);
    Ok(Json(
        comments_service::list_all(state.get_pool(), limit, offset).await?,
    ))
}

/// 刪除一則留言
async fn delete_comment(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::CommentDelete)?;
    comments_service::delete(state.get_pool(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
