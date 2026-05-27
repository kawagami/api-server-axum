use crate::{
    errors::AppError,
    repositories::{permissions as permissions_repo, redis, roles as roles_repo},
    state::AppState,
    structs::roles::{NewRole, Permission, Role, RoleWithPermissions, SetRolePermissions},
};

pub async fn get_roles(state: &AppState) -> Result<Vec<Role>, AppError> {
    roles_repo::get_roles(state).await
}

pub async fn get_role(state: &AppState, role_id: i32) -> Result<RoleWithPermissions, AppError> {
    roles_repo::get_role_with_permissions(state, role_id).await
}

pub async fn create_role(state: &AppState, new_role: NewRole) -> Result<Role, AppError> {
    roles_repo::create_role(state, &new_role).await
}

pub async fn set_role_permissions(
    state: &AppState,
    role_id: i32,
    body: SetRolePermissions,
) -> Result<(), AppError> {
    let emails = roles_repo::get_emails_by_role_id(state, role_id).await?;
    roles_repo::set_role_permissions(state, role_id, &body.permission_ids).await?;
    redis::invalidate_permissions_for_emails(state, &emails).await;
    Ok(())
}

pub async fn delete_role(state: &AppState, role_id: i32) -> Result<(), AppError> {
    let emails = roles_repo::get_emails_by_role_id(state, role_id).await?;
    roles_repo::delete_role(state, role_id).await?;
    redis::invalidate_permissions_for_emails(state, &emails).await;
    Ok(())
}

pub async fn get_permissions(state: &AppState) -> Result<Vec<Permission>, AppError> {
    permissions_repo::get_permissions(state).await
}
