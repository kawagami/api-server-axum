use crate::{
    errors::AppError,
    repositories::{redis, roles as roles_repo, users as users_repo},
    state::AppStateV2,
    structs::users::{NewUser, User},
};

pub async fn get_users(state: &AppStateV2) -> Result<Vec<User>, AppError> {
    users_repo::get_users(state).await
}

pub async fn create_user(state: &AppStateV2, user: NewUser) -> Result<(), AppError> {
    let role_id = roles_repo::get_role_id_by_name(state, "member").await?;
    users_repo::create_user(state, user, role_id).await
}

pub async fn set_user_roles(
    state: &AppStateV2,
    user_id: i64,
    role_ids: Vec<i32>,
) -> Result<(), AppError> {
    let email = users_repo::get_email_by_id(state, user_id).await?;
    users_repo::set_user_roles(state, user_id, &role_ids).await?;
    let _ = redis::del_user_permissions(state, &email).await;
    Ok(())
}
