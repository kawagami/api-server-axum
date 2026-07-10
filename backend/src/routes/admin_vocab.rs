use crate::{
    errors::{AppError, RequestError},
    repositories::vocab as vocab_repo,
    services::vocab_ja,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        pagination::PageQuery,
        roles::Perm,
        vocab::{AdminWordListQuery, AdminWordListResponse, UpdateWordRequest},
    },
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
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
    let (data, total) = vocab_repo::admin_list(state.get_pool(), &filter, limit, offset).await?;
    Ok(Json(AdminWordListResponse { data, total }))
}

/// 更新單字(釋義/讀音/難度/上下架;表記與語言不可改)
async fn update_word(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(mut req): Json<UpdateWordRequest>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::VocabUpdate)?;

    if !(1..=5).contains(&req.difficulty) {
        return Err(AppError::RequestError(RequestError::UnprocessableContent(
            "難度須在 1–5".to_string(),
        )));
    }
    let language = vocab_repo::admin_word_language(state.get_pool(), id)
        .await?
        .ok_or(AppError::RequestError(RequestError::NotFound))?;
    if language == "ja" {
        // 日文必有讀音(DB CHECK 也擋,這裡先給友善錯誤);
        // 主讀音必須在可接受讀音內,否則拼字題答主讀音會被判錯
        let reading = req.reading.as_deref().map(str::trim).unwrap_or_default();
        if reading.is_empty() {
            return Err(AppError::RequestError(RequestError::UnprocessableContent(
                "日文單字必須有讀音".to_string(),
            )));
        }
        let normalized = vocab_ja::normalize_reading(reading);
        let accepted = req.accepted_readings.get_or_insert_with(Vec::new);
        accepted.retain(|r| !r.trim().is_empty());
        if !accepted.iter().any(|r| vocab_ja::normalize_reading(r) == normalized) {
            accepted.insert(0, reading.to_string());
        }
    }

    vocab_repo::admin_update_word(state.get_pool(), id, &req).await?;
    Ok(StatusCode::NO_CONTENT)
}
