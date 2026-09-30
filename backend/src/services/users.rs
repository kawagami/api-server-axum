use crate::{
    errors::{AppError, AuthError},
    repositories::{redis, roles as roles_repo, users as users_repo},
    state::Settings,
    structs::{auth::AuthenticatedUser, roles::Role, users::{NewUser, User}},
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
    actor: &AuthenticatedUser,
    mut user: NewUser,
) -> Result<User, AppError> {
    let role_ids = if user.role_ids.is_empty() {
        default_role_ids(pool, settings).await?
    } else {
        std::mem::take(&mut user.role_ids)
    };
    // 預設角色那條路徑也要過 guard：`new_user_default_roles` 是 app_settings，
    // 只需 setting:update 就能改，不擋的話等於另開一條提權門。
    // 先驗再 hash：bcrypt 是百毫秒級的 CPU，沒必要為一個註定被拒的請求先燒掉。
    super::roles::ensure_assignable(pool, actor, &role_ids).await?;
    user.password = super::auth::hash_password(user.password).await?;
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

/// 刪管理員。**「不可刪自己」在這裡擋**（不是 route）：刪掉的瞬間這張 token 就不代表任何人，
/// 而且可能刪掉最後一個能登入的管理員。前端對自己那列不顯示刪除鈕，但那只是 UI；
/// 檢查跟寫入綁在同一支函式，之後多一個呼叫端也漏不掉（見 ARCHITECTURE.md「分層鐵律」）。
pub async fn delete_user(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    actor: &AuthenticatedUser,
    user_id: i64,
) -> Result<(), AppError> {
    if user_id == actor.id {
        return Err(AuthError::ForbiddenAction("不可刪除自己的帳號，請由其他管理員操作".to_string()).into());
    }
    users_repo::delete_user(pool, user_id).await?;
    redis::invalidate_user_identity(redis_pool, user_id).await;
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
    actor: &AuthenticatedUser,
    user_id: i64,
    role_ids: Vec<i32>,
) -> Result<(), AppError> {
    // 不可改自己的角色：否則有 role:assign 的人可以自行加掛任何角色（自我提權）。
    // 要調整自己的權限得請另一位管理員操作。
    if user_id == actor.id {
        return Err(AuthError::ForbiddenAction("不可變更自己的角色，請由其他管理員操作".to_string()).into());
    }
    super::roles::ensure_assignable(pool, actor, &role_ids).await?;
    users_repo::set_user_roles(pool, user_id, &role_ids).await?;
    redis::invalidate_user_identity(redis_pool, user_id).await;
    Ok(())
}
