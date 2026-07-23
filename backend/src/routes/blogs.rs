use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    middleware::{auth, rate_limit},
    services::blog_comments as comments_service,
    services::blogs as blogs_service,
    structs::blog_comments::{BlogComment, BlogCommentPaginatedResponse, NewComment},
    structs::blogs::{BlogFilter, BlogsResponse, DbBlog, TagCount},
    structs::members::AuthenticatedMember,
    structs::pagination::PageQuery,
    state::AppState,
};

pub fn new(state: AppState) -> Router<AppState> {
    // 留言提交:公開未認證寫入,掛 per-IP rate limit 防灌水 + optional member auth(帶 token 綁 member_id)
    let comment_post = Router::new()
        .route("/{id}/comments", post(create_comment))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize_member_optional,
        ))
        .layer(middleware::from_fn_with_state(
            state,
            rate_limit::comments_rate_limit,
        ));

    Router::new()
        .route("/", get(get_blogs))
        .route("/tags", get(get_tags))
        .route("/tags/counts", get(get_tag_counts))
        .route("/{id}", get(get_blog))
        .route("/{id}/comments", get(list_comments))
        .merge(comment_post)
}

async fn get_blogs(
    Query(page): Query<PageQuery>,
    Query(filter): Query<BlogFilter>,
    State(state): State<AppState>,
) -> Result<Json<BlogsResponse>, AppError> {
    let blogs = blogs_service::get_blogs(
        state.get_pool(),
        &page,
        filter.tag,
        filter.author,
        filter.q,
        filter.sort,
    )
    .await?;
    Ok(Json(blogs))
}

async fn get_tags(
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, AppError> {
    let tags = blogs_service::get_tags(state.get_pool()).await?;
    Ok(Json(tags))
}

async fn get_tag_counts(
    State(state): State<AppState>,
) -> Result<Json<Vec<TagCount>>, AppError> {
    let tags = blogs_service::get_tag_counts(state.get_pool()).await?;
    Ok(Json(tags))
}

async fn get_blog(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DbBlog>, AppError> {
    let blog = blogs_service::get_blog(state.get_pool(), id).await?;
    Ok(Json(blog))
}

/// 單篇 blog 的公開留言列表(舊到新)
async fn list_comments(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(page): Query<PageQuery>,
) -> Result<Json<BlogCommentPaginatedResponse>, AppError> {
    let (limit, offset) = page.to_limit_offset(50);
    Ok(Json(
        comments_service::list_by_blog(state.get_pool(), id, limit, offset).await?,
    ))
}

/// 提交一則留言。optional-auth 有帶有效 member token → 綁 member_id;否則為訪客(可帶自填名)
async fn create_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    member: Option<Extension<AuthenticatedMember>>,
    Json(input): Json<NewComment>,
) -> Result<(StatusCode, Json<BlogComment>), AppError> {
    let member_id = member.map(|m| m.0.member_id);
    let comment = comments_service::create(state.get_pool(), id, member_id, input).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}
