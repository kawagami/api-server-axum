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

pub async fn sign_in(
    pool: &Pool<Postgres>,
    redis_pool: &RedisPool<RedisConnectionManager>,
    jwt_secret: &str,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let db_user = users::check_email_exists(pool, email)
        .await
        .map_err(|_| AppError::AuthError(AuthError::UserNotFound))?;

    let user = CurrentUser {
        email: db_user.email,
        password_hash: db_user.password,
    };

    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
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

    if !verify_password(current_password, &db_user.password)? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
    }

    let new_hash = hash(new_password, DEFAULT_COST)
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼 hash 失敗".to_string())))?;

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

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證處理失敗".to_string())))
}
