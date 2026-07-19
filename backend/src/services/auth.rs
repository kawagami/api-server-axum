use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, roles as roles_repo, users},
    structs::auth::Claims,
};
use bb8::Pool as RedisPool;
use bb8_redis::RedisConnectionManager;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::{Pool, Postgres};
use std::sync::LazyLock;

// 帳號不存在時也要跑一次同 cost 的 bcrypt，拉平回應時間防 timing 枚舉。
// lazy 初始化發生在 spawn_blocking 執行緒，不佔 async worker。
static DUMMY_HASH: LazyLock<String> =
    LazyLock::new(|| hash("dummy-password", DEFAULT_COST).expect("bcrypt dummy hash"));

pub async fn sign_in(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    name: &str,
    password: &str,
) -> Result<String, AppError> {
    let (id, password_hash) = match users::get_credentials_by_name(pool, name).await? {
        Some(cred) => cred,
        None => {
            // 帳號不存在也跑一次 bcrypt，拉平回應時間防 timing 枚舉
            dummy_verify_password(password.to_string()).await;
            return Err(AppError::AuthError(AuthError::InvalidCredentials));
        }
    };

    if !verify_password(password.to_string(), password_hash).await? {
        return Err(AppError::AuthError(AuthError::InvalidCredentials));
    }

    complete_admin_login(pool, redis_pool, jwt_secret, id).await
}

/// 身分驗證通過後的共同收尾（密碼與 passkey 登入共用）：
/// Redis 寫 login key + 快取 permissions + 簽發 JWT。
pub async fn complete_admin_login(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    id: i64,
) -> Result<String, AppError> {
    let login_key = format!("user:login:{}", id);
    redis::redis_set(redis_pool, &login_key, &id.to_string()).await?;

    let permissions = roles_repo::get_user_permission_strings_by_id(pool, id).await?;
    redis::set_user_permissions(redis_pool, id, &permissions).await?;

    encode_jwt(id, jwt_secret)
}

pub async fn refresh_admin_token(
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    id: i64,
) -> Result<String, AppError> {
    let login_key = format!("user:login:{}", id);
    redis::redis_set(redis_pool, &login_key, &id.to_string()).await?;
    encode_jwt(id, jwt_secret)
}

pub async fn change_password(
    pool: &Pool<Postgres>,
    id: i64,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let current_hash = users::get_password_by_id(pool, id)
        .await?
        .ok_or(AppError::AuthError(AuthError::UserNotFound))?;

    if !verify_password(current_password.to_string(), current_hash).await? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
    }

    let new_hash = hash_password(new_password.to_string()).await?;

    users::update_password(pool, id, &new_hash).await
}

fn encode_jwt(id: i64, jwt_secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(1)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    // sub 存 user id（字串），與前台 member token 一致；顯示名 name 不進 token
    let claim = Claims { iat, exp, sub: id.to_string(), role: "admin".to_string() };

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::AuthError(AuthError::InvalidToken))
}

async fn dummy_verify_password(password: String) {
    let _ = tokio::task::spawn_blocking(move || verify(password, &DUMMY_HASH)).await;
}

// bcrypt 為 CPU-bound（DEFAULT_COST 約百毫秒），用 spawn_blocking 避免卡住 tokio worker
async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || verify(password, &hash))
        .await
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證 task 失敗".to_string())))?
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證處理失敗".to_string())))
}

pub(crate) async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || hash(password, DEFAULT_COST))
        .await
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼 hash task 失敗".to_string())))?
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼 hash 失敗".to_string())))
}
