use crate::extract::{Json, Path, Query};
use crate::{
    errors::AppError,
    services::vocab as vocab_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        pagination::PageQuery,
        roles::Perm,
        vocab::{AdminWordListQuery, AdminWordListResponse, UpdateWordRequest},
    },
};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    routing::{get, put},
    Router
};

pub fn new(state: AppState) -> Router<AppState> {
    super::with_auth(
        state,
        Router::new()
            .route("/words", get(list_words))
            .route("/words/{id}", put(update_word)),
    )
}

/// 題庫分頁列表(?language=&difficulty=&enabled=&q=&sort=wrong&page=&per_page=)
async fn list_words(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(filter): Query<AdminWordListQuery>,
    Query(page): Query<PageQuery>,
) -> Result<Json<AdminWordListResponse>, AppError> {
    auth_user.require_permission(Perm::VocabRead)?;
    let (limit, offset) = page.to_limit_offset(50);
    Ok(Json(
        vocab_service::admin_list_words(state.get_pool(), &filter, limit, offset).await?,
    ))
}

/// 更新單字(釋義/讀音/難度/上下架;表記與語言不可改)
async fn update_word(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(mut req): Json<UpdateWordRequest>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::VocabUpdate)?;
    vocab_service::admin_update_word(state.get_pool(), id, &mut req).await?;
    Ok(StatusCode::NO_CONTENT)
}
