use crate::{
    errors::AppError,
    repositories::{redis, users},
    state::AppStateV2,
    structs::auth::{Claims, CurrentUser, SignInData},
};
use axum::{
    body::Body,
    extract::{Json, Request, State},
    http,
    http::Response,
    middleware::Next,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
pub async fn authorize(
    State(state): State<AppStateV2>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    // 取得 header 中的 token
    let auth_header = req
        .headers_mut()
        .get(http::header::AUTHORIZATION)
        .ok_or(AppError::MissingToken)?
        .to_str()
        .map_err(|_| AppError::InvalidHeaderFormat)?;
    let mut header = auth_header.split_whitespace();
    let (_bearer, token) = (header.next(), header.next());

    let token_data = decode_jwt(token.ok_or(AppError::MissingToken)?.to_string())
        .map_err(|_| AppError::DecodeTokenFail)?;

    // 檢查 token 中的 email 是否存在於 redis
    let key = format!("user:login:{}", token_data.claims.email);
    let exists = redis::redis_check_key_exists(&state, &key)
        .await
        .map_err(|err| AppError::RedisError(err.to_string()))?;

    if !exists {
        return Err(AppError::UnauthorizedUser);
    }

    Ok(next.run(req).await)
}

pub async fn sign_in(
    State(state): State<AppStateV2>,
    Json(user_data): Json<SignInData>,
) -> Result<Json<String>, AppError> {
    // 檢查資料庫是否存在該 email
    let user = retrieve_user_by_email(&state, &user_data.email)
        .await
        .map_err(|_| AppError::UserNotFound)?;

    // 比對密碼是否吻合
    if !verify_password(&user_data.password, &user.password_hash)
        .map_err(|_| AppError::InternalError("密碼驗證處理失敗".into()))?
    {
        return Err(AppError::PasswordVerificationFailed);
    }

    // 在 redis 紀錄登入資訊
    let key = format!("user:login:{}", user.email);
    redis::redis_set(&state, &key, &user.email)
        .await
        .map_err(|err| AppError::RedisError(err.to_string()))?;

    // 生成合規 token
    let token = encode_jwt(user.email)?;

    // 返回 json 格式包裹的 token
    Ok(Json(token))
}

async fn retrieve_user_by_email(state: &AppStateV2, email: &str) -> Result<CurrentUser, AppError> {
    users::check_email_exists(state, email)
        .await
        .map(|db_user| CurrentUser {
            email: db_user.email,
            password_hash: db_user.password,
        })
        .map_err(|_| AppError::UserNotFound)
}

pub fn encode_jwt(email: String) -> Result<String, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::MissingEnvVariable("JWT_SECRET".to_string()))?;

    let now = Utc::now();
    let expire: chrono::TimeDelta = Duration::hours(1);
    let exp: usize = (now + expire).timestamp() as usize;
    let iat: usize = now.timestamp() as usize;

    let claim = Claims { iat, exp, email };

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::JwtEncodeFailed)
}

pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::MissingEnvVariable("JWT_SECRET".to_string()))?;

    decode(
        &jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::JwtExpired,
        _ => AppError::JwtDecodeFailed,
    })
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn _hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    let hash = hash(password, DEFAULT_COST)?;
    Ok(hash)
}
