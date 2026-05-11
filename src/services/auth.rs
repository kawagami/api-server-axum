use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, roles as roles_repo, users},
    state::AppState,
    structs::auth::{Claims, CurrentUser},
};
use bcrypt::verify;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

pub async fn sign_in(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<String, AppError> {
    let db_user = users::check_email_exists(state, email)
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
    redis::redis_set(state, &login_key, &user.email).await?;

    let permissions =
        roles_repo::get_user_permission_strings_by_email(state, &user.email).await?;
    redis::set_user_permissions(state, &user.email, &permissions).await?;

    encode_jwt(user.email)
}

pub fn refresh_admin_token(email: String) -> Result<String, AppError> {
    encode_jwt(email)
}

fn encode_jwt(email: String) -> Result<String, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::SystemError(SystemError::EnvVarMissing("JWT_SECRET".to_string())))?;

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
