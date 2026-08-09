use crate::{
    errors::{AppError, AuthError},
    repositories::{redis, roles as roles_repo, users as users_repo},
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
    let id = verify_admin_token(&state, token).await?;

    let permissions = match redis::get_user_permissions(state.get_redis_pool(), id).await? {
        Some(perms) => perms,
        None => {
            let perms =
                roles_repo::get_user_permission_strings_by_id(state.get_pool(), id).await?;
            // 回寫失敗不擋這次請求（權限已經拿到），但要留痕 —— 這條靜默失敗的症狀是
            // 「每個 /admin/* 請求都多一次 roles JOIN」，在 log 上完全看不出原因
            if let Err(e) = redis::set_user_permissions(state.get_redis_pool(), id, &perms).await {
                tracing::warn!("權限快取回寫失敗 user_id={}: {}", id, e);
            }
            perms
        }
    };

    // 取顯示名 + 是否 super_admin（帳號已刪 → 視為未授權）
    let (name, is_super_admin) = users_repo::get_identity_by_id(state.get_pool(), id)
        .await?
        .ok_or(AppError::AuthError(AuthError::Unauthorized))?;

    req.extensions_mut().insert(AuthenticatedUser {
        id,
        name,
        permissions,
        is_super_admin,
    });

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

/// 選擇性 member 驗證:有有效 member token 就塞入 `AuthenticatedMember`,
/// 沒有 / 無效一律放行(不擋),供「訪客也能用、登入才有額外功能」的端點使用。
pub async fn authorize_member_optional(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AppError> {
    if let Ok(token) = extract_token(&req) {
        if let Ok(token_data) = decode_jwt(token, &state.get_config().jwt_secret) {
            if token_data.claims.role == "member" {
                if let Ok(member_id) = token_data.claims.sub.parse::<i64>() {
                    req.extensions_mut()
                        .insert(AuthenticatedMember { member_id });
                }
            }
        }
    }
    Ok(next.run(req).await)
}

pub(crate) fn extract_token(req: &Request) -> Result<String, AppError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(AppError::AuthError(AuthError::MissingToken))?
        .to_str()
        .map_err(|_| AppError::AuthError(AuthError::InvalidHeader))?;

    let mut parts = auth_header.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(scheme), Some(token)) if scheme.eq_ignore_ascii_case("Bearer") && !token.is_empty() => {
            Ok(token.to_string())
        }
        _ => Err(AppError::AuthError(AuthError::InvalidHeader)),
    }
}

/// 驗證 admin JWT（簽章、role、Redis login session），回傳 user id。
/// middleware 與 WS 升級握手共用，JWT 驗證邏輯只此一份。
pub(crate) async fn verify_admin_token(state: &AppState, token: String) -> Result<i64, AppError> {
    let token_data = decode_jwt(token, &state.get_config().jwt_secret)?;

    if token_data.claims.role != "admin" {
        return Err(AppError::AuthError(AuthError::Forbidden));
    }

    let id: i64 = token_data
        .claims
        .sub
        .parse()
        .map_err(|_| AppError::AuthError(AuthError::InvalidToken))?;

    // 簽章有效不代表 session 還在：登出 / 改密碼會撤掉 `user:login:{id}`，
    // 撤銷後未過期的 token 必須立刻失效。
    if !redis::user_login_exists(state.get_redis_pool(), id).await? {
        return Err(AppError::AuthError(AuthError::Unauthorized));
    }
    Ok(id)
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
