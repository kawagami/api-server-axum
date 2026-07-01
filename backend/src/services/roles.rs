use crate::{
    errors::AppError,
    repositories::{permissions as permissions_repo, redis, roles as roles_repo},
    structs::roles::{NewRole, Permission, Role, RoleWithPermissions, SetRolePermissions},
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use sqlx::{Pool, Postgres};

pub async fn get_roles(pool: &Pool<Postgres>) -> Result<Vec<Role>, AppError> {
    roles_repo::get_roles(pool).await
}

pub async fn get_role(pool: &Pool<Postgres>, role_id: i32) -> Result<RoleWithPermissions, AppError> {
    roles_repo::get_role_with_permissions(pool, role_id).await
}

pub async fn create_role(pool: &Pool<Postgres>, new_role: NewRole) -> Result<Role, AppError> {
    roles_repo::create_role(pool, &new_role).await
}

pub async fn set_role_permissions(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    role_id: i32,
    body: SetRolePermissions,
) -> Result<(), AppError> {
    let emails = roles_repo::get_emails_by_role_id(pool, role_id).await?;
    roles_repo::set_role_permissions(pool, role_id, &body.permission_ids).await?;
    redis::invalidate_permissions_for_emails(redis_pool, &emails).await;
    Ok(())
}

pub async fn delete_role(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    role_id: i32,
) -> Result<(), AppError> {
    let emails = roles_repo::get_emails_by_role_id(pool, role_id).await?;
    roles_repo::delete_role(pool, role_id).await?;
    redis::invalidate_permissions_for_emails(redis_pool, &emails).await;
    Ok(())
}

pub async fn get_permissions(pool: &Pool<Postgres>) -> Result<Vec<Permission>, AppError> {
    permissions_repo::get_permissions(pool).await
}
