use crate::{
    errors::AppError,
    repositories::notes as notes_repo,
    state::AppState,
    structs::notes::{HackmdNoteListAndTag, Tag},
};

pub async fn get_tags(state: &AppState) -> Result<Vec<Tag>, AppError> {
    notes_repo::get_tags(state).await
}

pub async fn get_lists(state: &AppState) -> Result<Vec<HackmdNoteListAndTag>, AppError> {
    notes_repo::get_lists(state).await
}
