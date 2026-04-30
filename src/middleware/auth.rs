use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, roles as roles_repo},
    state::AppStateV2,
    structs::auth::{AuthenticatedUser, Claims},
};
use axum::{
    body::Body,
    extract::{Request, State},
    http,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

pub async fn authorize(
    State(state): State<AppStateV2>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let token_data = decode_jwt(token)?;

    let key = format!("user:login:{}", token_data.claims.email);
    verify_user_login(&state, &key).await?;

    Ok(next.run(req).await)
}

/// 驗證 token 並將 AuthenticatedUser（含 permissions）注入 request extensions。
/// 用於需要細粒度權限檢查的 route，取代 authorize middleware。
pub async fn authorize_and_load(
    State(state): State<AppStateV2>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let token_data = decode_jwt(token)?;
    let email = token_data.claims.email;

    let login_key = format!("user:login:{}", email);
    verify_user_login(&state, &login_key).await?;

    let permissions = match redis::get_user_permissions(&state, &email).await? {
        Some(perms) => perms,
        None => {
            // 快取失效：從 DB 重新載入並快取
            let perms =
                roles_repo::get_user_permission_strings_by_email(&state, &email).await?;
            let _ = redis::set_user_permissions(&state, &email, &perms).await;
            perms
        }
    };

    req.extensions_mut().insert(AuthenticatedUser { email, permissions });

    Ok(next.run(req).await)
}

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

async fn verify_user_login(state: &AppStateV2, key: &str) -> Result<(), AppError> {
    redis::redis_check_key_exists(state, key)
        .await?
        .then_some(())
        .ok_or(AppError::AuthError(AuthError::Unauthorized))
}

fn decode_jwt(jwt: String) -> Result<TokenData<Claims>, AppError> {
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
