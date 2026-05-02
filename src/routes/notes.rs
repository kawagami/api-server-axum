use crate::{
    errors::AppError,
    services::notes as notes_service,
    state::AppState,
    structs::notes::{HackmdNoteListAndTag, Tag},
};
use axum::{
    extract::{Json, State},
    routing::get,
    Router,
};

pub fn new() -> Router<AppState> {
    Router::new()
        .route("/lists", get(get_lists))
        .route("/tags", get(get_tags))
}

async fn get_tags(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, AppError> {
    Ok(Json(notes_service::get_tags(&state).await?))
}

async fn get_lists(
    State(state): State<AppState>,
) -> Result<Json<Vec<HackmdNoteListAndTag>>, AppError> {
    Ok(Json(notes_service::get_lists(&state).await?))
}
