use crate::{
    errors::{AppError, AuthError},
    repositories::{permissions as permissions_repo, redis, roles as roles_repo},
    structs::roles::{NewRole, Permission, Role, RoleWithPermissions, SetRolePermissions},
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use sqlx::{Pool, Postgres};

/// 指派角色前的守門：任何人都不得把 `super_admin` 指派出去。
///
/// 這條的存在理由：內建 `admin` 角色（baseline 的 role 3「後台全功能」）拿到的是
/// `user:read/create/update/delete`，**刻意沒有** `role:*`——「管帳號」與「管角色」
/// 是分開的兩層。但建立管理員的 API 收 `role_ids`，若不在這裡擋，一般管理員只要
/// 有 `user:create` 就能建一個 super_admin 帳號再登入，整條邊界就沒了。
///
/// 呼叫點：`services/users.rs` 的 create_user 與 set_user_roles（含讀 app_settings
/// 預設角色那條 fallback 路徑）。
pub async fn ensure_assignable(pool: &Pool<Postgres>, role_ids: &[i32]) -> Result<(), AppError> {
    if roles_repo::contains_super_admin(pool, role_ids).await? {
        return Err(AuthError::Forbidden.into());
    }
    Ok(())
}

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
    let ids = roles_repo::get_ids_by_role_id(pool, role_id).await?;
    roles_repo::set_role_permissions(pool, role_id, &body.permission_ids).await?;
    redis::invalidate_permissions_for_ids(redis_pool, &ids).await;
    Ok(())
}

pub async fn delete_role(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    role_id: i32,
) -> Result<(), AppError> {
    let ids = roles_repo::get_ids_by_role_id(pool, role_id).await?;
    roles_repo::delete_role(pool, role_id).await?;
    redis::invalidate_permissions_for_ids(redis_pool, &ids).await;
    Ok(())
}

pub async fn get_permissions(pool: &Pool<Postgres>) -> Result<Vec<Permission>, AppError> {
    permissions_repo::get_permissions(pool).await
}
