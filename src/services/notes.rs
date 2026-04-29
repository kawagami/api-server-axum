use crate::{
    errors::AppError,
    repositories::notes as notes_repo,
    state::AppStateV2,
    structs::notes::{HackmdNoteListAndTag, Tag},
};

pub async fn get_tags(state: &AppStateV2) -> Result<Vec<Tag>, AppError> {
    notes_repo::get_tags(state).await
}

pub async fn get_lists(state: &AppStateV2) -> Result<Vec<HackmdNoteListAndTag>, AppError> {
    notes_repo::get_lists(state).await
}
