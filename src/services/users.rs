use crate::{
    errors::AppError,
    repositories::{redis, roles as roles_repo, users as users_repo},
    state::AppState,
    structs::{roles::Role, users::{NewUser, User}},
};

pub async fn get_users(state: &AppState) -> Result<Vec<User>, AppError> {
    users_repo::get_users(state).await
}

pub async fn create_user(state: &AppState, user: NewUser) -> Result<(), AppError> {
    let role_id = roles_repo::get_role_id_by_name(state, "member").await?;
    users_repo::create_user(state, user, role_id).await
}

pub async fn delete_user(state: &AppState, user_id: i64) -> Result<(), AppError> {
    let email = users_repo::delete_user(state, user_id).await?;
    let _ = redis::del_user_permissions(state, &email).await;
    let _ = redis::del_user_login(state, &email).await;
    Ok(())
}

pub async fn get_user_roles(state: &AppState, user_id: i64) -> Result<Vec<Role>, AppError> {
    users_repo::get_user_roles(state, user_id).await
}

pub async fn set_user_roles(
    state: &AppState,
    user_id: i64,
    role_ids: Vec<i32>,
) -> Result<(), AppError> {
    let email = users_repo::get_email_by_id(state, user_id).await?;
    users_repo::set_user_roles(state, user_id, &role_ids).await?;
    let _ = redis::del_user_permissions(state, &email).await;
    Ok(())
}
