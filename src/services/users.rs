use crate::{
    errors::AppError,
    repositories::{redis, roles as roles_repo, users as users_repo},
    structs::{roles::Role, users::{NewUser, User}},
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use sqlx::{Pool, Postgres};

pub async fn get_users(pool: &Pool<Postgres>) -> Result<Vec<User>, AppError> {
    users_repo::get_users(pool).await
}

pub async fn create_user(pool: &Pool<Postgres>, mut user: NewUser) -> Result<(), AppError> {
    user.password = super::auth::hash_password(user.password).await?;
    let role_id = roles_repo::get_role_id_by_name(pool, "member").await?;
    users_repo::create_user(pool, user, role_id).await
}

pub async fn delete_user(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    user_id: i64,
) -> Result<(), AppError> {
    let email = users_repo::delete_user(pool, user_id).await?;
    if let Err(e) = redis::del_user_permissions(redis_pool, &email).await {
        tracing::warn!("Failed to invalidate permissions cache for {}: {}", email, e);
    }
    if let Err(e) = redis::del_user_login(redis_pool, &email).await {
        tracing::warn!("Failed to invalidate login cache for {}: {}", email, e);
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
    let email = users_repo::get_email_by_id(pool, user_id).await?;
    users_repo::set_user_roles(pool, user_id, &role_ids).await?;
    if let Err(e) = redis::del_user_permissions(redis_pool, &email).await {
        tracing::warn!("Failed to invalidate permissions cache for {}: {}", email, e);
    }
    Ok(())
}
