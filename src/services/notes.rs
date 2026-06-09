use crate::{
    errors::AppError,
    repositories::notes as notes_repo,
    structs::notes::{HackmdNoteListAndTag, Tag},
};
use sqlx::{Pool, Postgres};

pub async fn get_tags(pool: &Pool<Postgres>) -> Result<Vec<Tag>, AppError> {
    notes_repo::get_tags(pool).await
}

pub async fn get_lists(pool: &Pool<Postgres>) -> Result<Vec<HackmdNoteListAndTag>, AppError> {
    notes_repo::get_lists(pool).await
}
