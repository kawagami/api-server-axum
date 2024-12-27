use crate::{
    repositories::notes,
    state::AppStateV2,
    structs::notes::{HackmdNoteListAndTag, Tag},
};
use axum::extract::{Json, State};

pub async fn get_all_note_list_tags(State(state): State<AppStateV2>) -> Json<Vec<Tag>> {
    let response = notes::get_all_note_list_tags(&state).await;

    match response {
        Ok(tags) => Json(tags),
        Err(err) => {
            tracing::error!("{}", err);
            Json(vec![])
        }
    }
}

pub async fn get_all_note_lists(
    State(state): State<AppStateV2>,
) -> Json<Vec<HackmdNoteListAndTag>> {
    let response = notes::get_all_note_lists(&state).await;

    match response {
        Ok(tags) => Json(tags),
        Err(err) => {
            tracing::error!("{}", err);
            Json(vec![])
        }
    }
}
