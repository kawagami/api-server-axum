use crate::{
    errors::{AppError, AuthError, SystemError},
    repositories::{redis, roles as roles_repo},
    state::AppState,
    structs::{
        auth::{AuthenticatedUser, Claims},
        members::AuthenticatedMember,
    },
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
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let token_data = decode_jwt(token)?;

    if token_data.claims.role != "admin" {
        return Err(AppError::AuthError(AuthError::Forbidden));
    }

    let key = format!("user:login:{}", token_data.claims.sub);
    verify_user_login(&state, &key).await?;

    Ok(next.run(req).await)
}

pub async fn authorize_and_load(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let token_data = decode_jwt(token)?;

    if token_data.claims.role != "admin" {
        return Err(AppError::AuthError(AuthError::Forbidden));
    }

    let email = token_data.claims.sub;
    let login_key = format!("user:login:{}", email);
    verify_user_login(&state, &login_key).await?;

    let permissions = match redis::get_user_permissions(&state, &email).await? {
        Some(perms) => perms,
        None => {
            let perms =
                roles_repo::get_user_permission_strings_by_email(&state, &email).await?;
            let _ = redis::set_user_permissions(&state, &email, &perms).await;
            perms
        }
    };

    req.extensions_mut().insert(AuthenticatedUser { email, permissions });

    Ok(next.run(req).await)
}

pub async fn authorize_member(
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let token_data = decode_jwt(token)?;

    if token_data.claims.role != "member" {
        return Err(AppError::AuthError(AuthError::Forbidden));
    }

    let member_id: i64 = token_data
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::AuthError(AuthError::InvalidToken))?;

    req.extensions_mut().insert(AuthenticatedMember { member_id });

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

async fn verify_user_login(state: &AppState, key: &str) -> Result<(), AppError> {
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
