use crate::{
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

pub async fn get_tags(State(state): State<AppStateV2>) -> Json<Vec<Tag>> {
    let response = notes::get_tags(&state).await;

    match response {
        Ok(tags) => Json(tags),
        Err(err) => {
            tracing::error!("{}", err);
            Json(vec![])
        }
    }
}

pub async fn get_lists(
    State(state): State<AppStateV2>,
) -> Json<Vec<HackmdNoteListAndTag>> {
    let response = notes::get_lists(&state).await;

    match response {
        Ok(tags) => Json(tags),
        Err(err) => {
            tracing::error!("{}", err);
            Json(vec![])
        }
    }
}
