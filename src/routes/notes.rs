use crate::{
    errors::AppError,
    repositories::notes,
    state::AppStateV2,
    structs::notes::{HackmdNoteListAndTag, Tag},
};
use axum::{
    extract::{Json, State},
    routing::get,
    Router,
};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/lists", get(get_lists))
        .route("/tags", get(get_tags))
}

pub async fn get_tags(State(state): State<AppStateV2>) -> Result<Json<Vec<Tag>>, AppError> {
    let tags = notes::get_tags(&state).await?; // 自動傳播錯誤
    Ok(Json(tags))
}

pub async fn get_lists(
    State(state): State<AppStateV2>,
) -> Result<Json<Vec<HackmdNoteListAndTag>>, AppError> {
    let lists = notes::get_lists(&state).await?; // 自動傳播錯誤
    Ok(Json(lists))
}
