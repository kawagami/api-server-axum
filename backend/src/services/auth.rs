use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, roles as roles_repo, users},
    structs::auth::{Claims, CurrentUser},
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
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let db_user = match users::check_email_exists(pool, email).await {
        Ok(user) => user,
        Err(_) => {
            dummy_verify_password(password.to_string()).await;
            return Err(AppError::AuthError(AuthError::InvalidCredentials));
        }
    };

    let user = CurrentUser {
        email: db_user.email,
        password_hash: db_user.password,
    };

    if !verify_password(password.to_string(), user.password_hash.clone()).await? {
        return Err(AppError::AuthError(AuthError::InvalidCredentials));
    }

    let login_key = format!("user:login:{}", user.email);
    redis::redis_set(redis_pool, &login_key, &user.email).await?;

    let permissions =
        roles_repo::get_user_permission_strings_by_email(pool, &user.email).await?;
    redis::set_user_permissions(redis_pool, &user.email, &permissions).await?;

    encode_jwt(user.email, jwt_secret)
}

pub async fn refresh_admin_token(
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    email: String,
) -> Result<String, AppError> {
    let login_key = format!("user:login:{}", email);
    redis::redis_set(redis_pool, &login_key, &email).await?;
    encode_jwt(email, jwt_secret)
}

pub async fn change_password(
    pool: &Pool<Postgres>,
    email: &str,
    current_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let db_user = users::check_email_exists(pool, email)
        .await
        .map_err(|_| AppError::AuthError(AuthError::UserNotFound))?;

    if !verify_password(current_password.to_string(), db_user.password).await? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
    }

    let new_hash = hash_password(new_password.to_string()).await?;

    users::update_password(pool, email, &new_hash).await
}

fn encode_jwt(email: String, jwt_secret: &str) -> Result<String, AppError> {
    let now = Utc::now();
    let exp = (now + Duration::hours(1)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claims { iat, exp, sub: email, role: "admin".to_string() };

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
