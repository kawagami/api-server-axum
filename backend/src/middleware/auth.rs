use crate::{
    errors::{AppError, AuthError},
    repositories::redis,
    services::auth as auth_service,
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

    // 顯示名 / super_admin / 權限一次取齊（快取命中 = 零 DB）；
    // None = 帳號已刪但 token/session 未過期 → 視為未授權
    let identity = auth_service::load_identity(state.get_pool(), state.get_redis_pool(), id)
        .await?
        .ok_or(AppError::AuthError(AuthError::Unauthorized))?;

    req.extensions_mut().insert(AuthenticatedUser {
        id,
        name: identity.name,
        permissions: identity.permissions,
        is_super_admin: identity.is_super_admin,
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use chrono::{Duration, Utc};
    use http::HeaderValue;
    use jsonwebtoken::{encode, EncodingKey, Header};

    const SECRET: &str = "test-secret";

    fn req_with(value: HeaderValue) -> Request {
        Request::builder()
            .header(http::header::AUTHORIZATION, value)
            .body(Body::empty())
            .expect("建立測試 request")
    }

    fn req_auth(value: &str) -> Request {
        req_with(HeaderValue::from_str(value).expect("header 值"))
    }

    fn token_expiring(delta: Duration) -> String {
        let now = Utc::now();
        let claims = Claims {
            iat: now.timestamp() as usize,
            exp: (now + delta).timestamp() as usize,
            sub: "42".to_string(),
            role: "admin".to_string(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .expect("簽發測試 token")
    }

    #[test]
    fn extracts_bearer_token_case_insensitively() {
        assert_eq!(extract_token(&req_auth("Bearer abc.def.ghi")).unwrap(), "abc.def.ghi");
        assert_eq!(extract_token(&req_auth("bearer abc.def.ghi")).unwrap(), "abc.def.ghi");
        // 多餘空白由 split_whitespace 吸收
        assert_eq!(extract_token(&req_auth("Bearer   abc.def.ghi")).unwrap(), "abc.def.ghi");
    }

    /// 沒帶 header 與帶了壞 header 必須是不同錯誤：前端靠 401 的訊息決定
    /// 「導去登入頁」還是「這個請求本身寫錯了」
    #[test]
    fn missing_header_is_distinct_from_malformed_one() {
        let req = Request::builder().body(Body::empty()).unwrap();
        assert!(matches!(
            extract_token(&req),
            Err(AppError::AuthError(AuthError::MissingToken))
        ));
        assert!(extract_token(&req_auth("Bearer abc")).is_ok());
    }

    /// 這幾種都不該被當成有效憑證放進 decode_jwt
    #[test]
    fn malformed_authorization_headers_are_rejected() {
        for bad in ["Bearer", "Bearer ", "abc.def.ghi", "Basic abc", "BearerX abc", ""] {
            assert!(
                matches!(
                    extract_token(&req_auth(bad)),
                    Err(AppError::AuthError(AuthError::InvalidHeader))
                ),
                "{bad:?} 應被擋下"
            );
        }
        // 非 UTF-8 的 header 值（to_str 失敗）
        let raw = HeaderValue::from_bytes(&[0x42, 0xff]).expect("非 UTF-8 header");
        assert!(matches!(
            extract_token(&req_with(raw)),
            Err(AppError::AuthError(AuthError::InvalidHeader))
        ));
    }

    #[test]
    fn valid_token_decodes_to_its_claims() {
        let data = decode_jwt(token_expiring(Duration::hours(1)), SECRET).expect("應解得開");
        assert_eq!(data.claims.sub, "42");
        assert_eq!(data.claims.role, "admin");
    }

    /// 過期必須映射成 TokenExpired 而非 InvalidToken —— 前端靠這個分辨
    /// 「拿 refresh token 換一張」與「這張根本是偽造的，直接登出」。
    /// 兩者都是 401，映射寫反不會有任何徵兆，只會讓續期流程整個失效。
    #[test]
    fn expired_token_maps_to_token_expired() {
        // Validation::default() 有 60 秒 leeway，要退得夠遠才算過期
        let expired = token_expiring(-Duration::hours(1));
        assert!(matches!(
            decode_jwt(expired, SECRET),
            Err(AppError::AuthError(AuthError::TokenExpired))
        ));
    }

    #[test]
    fn wrong_secret_and_garbage_map_to_invalid_token() {
        let token = token_expiring(Duration::hours(1));
        assert!(matches!(
            decode_jwt(token.clone(), "another-secret"),
            Err(AppError::AuthError(AuthError::InvalidToken))
        ));
        assert!(matches!(
            decode_jwt("not-a-jwt".to_string(), SECRET),
            Err(AppError::AuthError(AuthError::InvalidToken))
        ));
        // 簽章被動過一個字元
        let mut tampered = token;
        tampered.pop();
        tampered.push('x');
        assert!(matches!(
            decode_jwt(tampered, SECRET),
            Err(AppError::AuthError(AuthError::InvalidToken))
        ));
    }
}
