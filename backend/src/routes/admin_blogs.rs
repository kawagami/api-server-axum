use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::{AppError, RequestError},
    repositories::blogs as blogs_repo,
    services::blogs as blogs_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        blogs::{BlogsResponse, PutBlog},
        pagination::PageQuery,
        roles::Perm,
        ws::WsEvent,
    },
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/", get(list_blogs))
            .route("/{id}", put(put_blog).delete(delete_blog)),
    )
}

/// 後台管理列表：一般 admin 只列自己的文章，super_admin 看全部（公開站台的 GET /blogs/ 不受影響）。
async fn list_blogs(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(page): Query<PageQuery>,
) -> Result<Json<BlogsResponse>, AppError> {
    auth_user.require_permission(Perm::BlogRead)?;
    Ok(Json(
        blogs_service::get_admin_blogs(state.get_pool(), auth_user.owner_filter(), &page).await?,
    ))
}

async fn put_blog(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(blog): Json<PutBlog>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::BlogUpdate)?;
    // 既有文章只能改自己的（super_admin 例外）；不存在＝新建，擁有者記為自己
    if let Some(author) = blogs_repo::get_author(state.get_pool(), id).await? {
        auth_user.require_owner(author)?;
    }
    let title = blogs_service::upsert_blog(state.get_pool(), id, blog, auth_user.id).await?;
    state.broadcast(WsEvent::BlogCreated, serde_json::json!({ "id": id, "title": title }));
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_blog(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::BlogDelete)?;
    let author = blogs_repo::get_author(state.get_pool(), id)
        .await?
        .ok_or(RequestError::NotFound)?;
    auth_user.require_owner(author)?;
    blogs_service::delete_blog_with_images(state.get_pool(), id).await?;
    Ok(StatusCode::NO_CONTENT)
}
