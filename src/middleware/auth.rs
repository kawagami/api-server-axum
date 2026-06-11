use crate::{
    errors::{AppError, AuthError},
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

pub async fn authorize_and_load(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let email = verify_admin_token(&state, token).await?;

    let permissions = match redis::get_user_permissions(state.get_redis_pool(), &email).await? {
        Some(perms) => perms,
        None => {
            let perms =
                roles_repo::get_user_permission_strings_by_email(state.get_pool(), &email).await?;
            let _ = redis::set_user_permissions(state.get_redis_pool(), &email, &perms).await;
            perms
        }
    };

    req.extensions_mut().insert(AuthenticatedUser { email, permissions });

    Ok(next.run(req).await)
}

pub async fn authorize_member(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    let token = extract_token(&req)?;
    let token_data = decode_jwt(token, &state.get_config().jwt_secret)?;

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

pub(crate) fn extract_token(req: &Request) -> Result<String, AppError> {
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

/// 驗證 admin JWT（簽章、role、Redis login session），回傳 email。
/// middleware 與 WS 升級握手共用，JWT 驗證邏輯只此一份。
pub(crate) async fn verify_admin_token(state: &AppState, token: String) -> Result<String, AppError> {
    let token_data = decode_jwt(token, &state.get_config().jwt_secret)?;

    if token_data.claims.role != "admin" {
        return Err(AppError::AuthError(AuthError::Forbidden));
    }

    let email = token_data.claims.sub;
    let login_key = format!("user:login:{}", email);
    verify_user_login(state, &login_key).await?;
    Ok(email)
}

async fn verify_user_login(state: &AppState, key: &str) -> Result<(), AppError> {
    redis::redis_check_key_exists(state.get_redis_pool(), key)
        .await?
        .then_some(())
        .ok_or(AppError::AuthError(AuthError::Unauthorized))
}

pub(crate) fn decode_jwt(jwt: String, secret: &str) -> Result<TokenData<Claims>, AppError> {
    decode(
        &jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
            AppError::AuthError(AuthError::TokenExpired)
        }
        _ => AppError::AuthError(AuthError::InvalidToken),
    })
}
