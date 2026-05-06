use crate::{
    errors::AppError,
    repositories::members as members_repo,
    state::AppState,
    structs::members::{Member, MemberDetail},
};

pub async fn get_members(state: &AppState) -> Result<Vec<Member>, AppError> {
    members_repo::get_members(state).await
}

pub async fn get_member_by_id(state: &AppState, id: i64) -> Result<Option<MemberDetail>, AppError> {
    members_repo::get_member_by_id(state, id).await
}
