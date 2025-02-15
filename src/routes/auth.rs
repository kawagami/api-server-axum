use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, users},
    state::AppStateV2,
    structs::auth::{Claims, CurrentUser, SignInData},
};
use axum::{
    body::Body,
    extract::{Json, Request, State},
    http::{self, Response},
    middleware::Next,
    routing::post,
    Router,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

pub fn new() -> Router<AppStateV2> {
    Router::new().route("/", post(sign_in))
}

pub async fn authorize(
    State(state): State<AppStateV2>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&mut req)?;
    let token_data = decode_jwt(token)?;

    let key = format!("user:login:{}", token_data.claims.email);
    verify_user_login(&state, &key).await?;

    Ok(next.run(req).await)
}

// 抽取 token 解析邏輯
fn extract_token(req: &Request) -> Result<String, AppError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(AppError::AuthError(AuthError::MissingToken))?
        .to_str()
        .map_err(|_| AppError::AuthError(AuthError::InvalidHeader))?;

    auth_header
        .split_whitespace()
        .nth(1)
        .ok_or(AppError::AuthError(AuthError::MissingToken))
        .map(ToString::to_string)
}

// 抽取 Redis 驗證邏輯
async fn verify_user_login(state: &AppStateV2, key: &str) -> Result<(), AppError> {
    redis::redis_check_key_exists(state, key)
        .await
        .map_err(|err| AppError::SystemError(SystemError::RedisError(err.to_string())))?
        .then_some(())
        .ok_or(AppError::AuthError(AuthError::Unauthorized))
}

pub async fn sign_in(
    State(state): State<AppStateV2>,
    Json(user_data): Json<SignInData>,
) -> Result<Json<String>, AppError> {
    let user = retrieve_user_by_email(&state, &user_data.email).await?;

    if !verify_password(&user_data.password, &user.password_hash)? {
        return Err(AppError::AuthError(AuthError::InvalidPassword));
    }

    let key = format!("user:login:{}", user.email);
    redis::redis_set(&state, &key, &user.email)
        .await
        .map_err(|err| AppError::SystemError(SystemError::RedisError(err.to_string())))?;

    let token = encode_jwt(user.email)?;

    Ok(Json(token))
}

async fn retrieve_user_by_email(state: &AppStateV2, email: &str) -> Result<CurrentUser, AppError> {
    users::check_email_exists(state, email)
        .await
        .map(|db_user| CurrentUser {
            email: db_user.email,
            password_hash: db_user.password,
        })
        .map_err(|_| AppError::AuthError(AuthError::UserNotFound))
}

pub fn encode_jwt(email: String) -> Result<String, AppError> {
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

pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::SystemError(SystemError::EnvVarMissing("JWT_SECRET".to_string())))?;

    decode(
        &jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::AuthError(AuthError::TokenExpired)
        }
        _ => AppError::AuthError(AuthError::InvalidToken),
    })
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼驗證處理失敗".to_string())))
}

pub fn _hash_password(password: &str) -> Result<String, AppError> {
    hash(password, DEFAULT_COST)
        .map_err(|_| AppError::SystemError(SystemError::Internal("密碼哈希失敗".to_string())))
}
