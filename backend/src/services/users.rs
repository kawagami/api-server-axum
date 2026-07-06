use crate::{
    errors::AppError,
    repositories::{redis, roles as roles_repo, users as users_repo},
    state::Settings,
    structs::{roles::Role, users::{NewUser, User}},
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use sqlx::{Pool, Postgres};

pub async fn get_users(pool: &Pool<Postgres>) -> Result<Vec<User>, AppError> {
    users_repo::get_users(pool).await
}

pub async fn create_user(
    pool: &Pool<Postgres>,
    settings: &Settings,
    mut user: NewUser,
) -> Result<(), AppError> {
    user.password = super::auth::hash_password(user.password).await?;
    let role_ids = if user.role_ids.is_empty() {
        default_role_ids(pool, settings).await?
    } else {
        std::mem::take(&mut user.role_ids)
    };
    users_repo::create_user(pool, user, &role_ids).await
}

/// 讀 app_settings `new_user_default_roles`（逗號分隔角色名稱）解析成角色 id；
/// 未設定 / 名稱都不存在時回空陣列（建立無角色管理員，之後再指派）
async fn default_role_ids(pool: &Pool<Postgres>, settings: &Settings) -> Result<Vec<i32>, AppError> {
    let names: Vec<String> = settings
        .get("new_user_default_roles")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if names.is_empty() {
        return Ok(Vec::new());
    }
    roles_repo::get_role_ids_by_names(pool, &names).await
}

pub async fn delete_user(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    user_id: i64,
) -> Result<(), AppError> {
    users_repo::delete_user(pool, user_id).await?;
    redis::invalidate_user_permissions(redis_pool, user_id).await;
    if let Err(e) = redis::del_user_login(redis_pool, user_id).await {
        tracing::warn!("Failed to invalidate login cache for user {}: {}", user_id, e);
    }
    Ok(())
}

pub async fn get_user_roles(pool: &Pool<Postgres>, user_id: i64) -> Result<Vec<Role>, AppError> {
    users_repo::get_user_roles(pool, user_id).await
}

pub async fn set_user_roles(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    user_id: i64,
    role_ids: Vec<i32>,
) -> Result<(), AppError> {
    users_repo::set_user_roles(pool, user_id, &role_ids).await?;
    redis::invalidate_user_permissions(redis_pool, user_id).await;
    Ok(())
}
