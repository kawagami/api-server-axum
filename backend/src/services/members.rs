use crate::{
    errors::AppError,
    repositories::members as members_repo,
    structs::{members::{Member, MemberDetail}, pagination::Paginated},
};
use sqlx::{Pool, Postgres};

pub async fn get_members(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<Paginated<Member>, AppError> {
    // count 與 list 併發跑：序列 await 是白吃一倍延遲（範本同 services/logs.rs）
    let (data, total) = tokio::try_join!(
        members_repo::get_members(pool, limit, offset),
        members_repo::count_members(pool),
    )?;
    Ok(Paginated::new(data, total))
}

pub async fn get_member_by_id(pool: &Pool<Postgres>, id: i64) -> Result<Option<MemberDetail>, AppError> {
    members_repo::get_member_by_id(pool, id).await
}
