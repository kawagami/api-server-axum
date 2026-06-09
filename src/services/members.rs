use crate::{
    errors::AppError,
    repositories::members as members_repo,
    structs::members::{Member, MemberDetail},
};
use sqlx::{Pool, Postgres};

pub async fn get_members(pool: &Pool<Postgres>) -> Result<Vec<Member>, AppError> {
    members_repo::get_members(pool).await
}

pub async fn get_member_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<MemberDetail>, AppError> {
    members_repo::get_member_by_id(pool, id).await
}
