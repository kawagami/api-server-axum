use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, users},
    state::AppStateV2,
    structs::auth::{Claims, CurrentUser, PasswordInput, SignInData},
};
use axum::{
    extract::{Json, State},
    routing::post,
    Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/", post(sign_in))
        .route("/hash", post(hash_password_handler))
}

// 處理用戶登入邏輯
pub async fn sign_in(
    State(state): State<AppStateV2>,
    Json(user_data): Json<SignInData>,
) -> Result<Json<String>, AppError> {
    let user = retrieve_user_by_email(&state, &user_data.email).await?; // 透過 Email 查詢用戶

    if !verify_password(&user_data.password, &user.password_hash)? {
        return Err(AppError::AuthError(AuthError::InvalidPassword)); // 驗證密碼失敗
    }

    let key = format!("user:login:{}", user.email);
    redis::redis_set(&state, &key, &user.email).await?;

    let token = encode_jwt(user.email)?; // 生成 JWT token

    Ok(Json(token))
}

// 透過 Email 查詢用戶
async fn retrieve_user_by_email(state: &AppStateV2, email: &str) -> Result<CurrentUser, AppError> {
    users::check_email_exists(state, email)
        .await
        .map(|db_user| CurrentUser {
            email: db_user.email,
            password_hash: db_user.password,
        })
        .map_err(|_| AppError::AuthError(AuthError::UserNotFound))
}

// 生成 JWT token
pub fn encode_jwt(email: String) -> Result<String, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::SystemError(SystemError::EnvVarMissing("JWT_SECRET".to_string())))?;

    let now = Utc::now();
    let exp = (now + Duration::hours(1)).timestamp() as usize; // 設定 1 小時後過期
    let iat = now.timestamp() as usize;

    let claim = Claims { iat, exp, email };

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::AuthError(AuthError::InvalidToken))
}

// 驗證用戶密碼
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證處理失敗".to_string())))
}

pub async fn hash_password_handler(
    Json(input): Json<PasswordInput>,
) -> Result<Json<String>, AppError> {
    let hashed = hash_password(&input.password)?;
    Ok(Json(hashed))
}

// 哈希用戶密碼
pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST)
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼哈希失敗".to_string())))
}
