use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, users},
    state::AppStateV2,
    structs::auth::{Claims, CurrentUser},
};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

pub async fn sign_in(
    state: &AppStateV2,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let user = users::check_email_exists(state, email)
        .await
        .map(|db_user| CurrentUser {
            email: db_user.email,
            password_hash: db_user.password,
        })
        .map_err(|_| AppError::AuthError(AuthError::UserNotFound))?;

    if !verify_password(password, &user.password_hash)? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
    }

    let key = format!("user:login:{}", user.email);
    redis::redis_set(state, &key, &user.email).await?;

    encode_jwt(user.email)
}

fn encode_jwt(email: String) -> Result<String, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::SystemError(SystemError::EnvVarMissing("JWT_SECRET".to_string())))?;

    let now = Utc::now();
    let exp = (now + Duration::hours(1)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claim = Claims { iat, exp, email };

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
