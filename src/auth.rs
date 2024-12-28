use crate::{
    repositories::users,
    state::AppStateV2,
    structs::auth::{AuthError, Claims, CurrentUser, SignInData},
};
use axum::{
    body::Body,
    extract::{Json, Request, State},
    http,
    http::{Response, StatusCode},
    middleware::Next,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

pub async fn authorize(
    State(state): State<AppStateV2>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AuthError> {
    // 取得 header 中的 token
    let auth_header = req.headers_mut().get(http::header::AUTHORIZATION);
    let auth_header = match auth_header {
        Some(header) => header.to_str().map_err(|_| AuthError {
            message: "Empty header is not allowed".to_string(),
            status_code: StatusCode::FORBIDDEN,
        })?,
        None => {
            return Err(AuthError {
                message: "Please add the JWT token to the header".to_string(),
                status_code: StatusCode::FORBIDDEN,
            })
        }
    };
    let mut header = auth_header.split_whitespace();
    let (_bearer, token) = (header.next(), header.next());

    // 解密 token
    let token_data = match decode_jwt(token.unwrap().to_string()) {
        Ok(data) => data,
        Err(_) => {
            return Err(AuthError {
                message: "Unable to decode token".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })
        }
    };

    // 檢查 token 中的 email 是否存在於資料庫
    let current_user = match retrieve_user_by_email(&state, &token_data.claims.email).await {
        Some(user) => user,
        None => {
            return Err(AuthError {
                message: "You are not an authorized user".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })
        }
    };

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}

pub async fn sign_in(
    State(state): State<AppStateV2>,
    Json(user_data): Json<SignInData>,
) -> Result<Json<String>, StatusCode> {
    // 檢查資料庫是否存在該 email
    let user = match retrieve_user_by_email(&state, &user_data.email).await {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // 比對密碼是否吻合
    if !verify_password(&user_data.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 生成合規 token
    let token = encode_jwt(user.email).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 返回 json 格式包裹的 token
    Ok(Json(token))
}

async fn retrieve_user_by_email(state: &AppStateV2, email: &str) -> Option<CurrentUser> {
    // 呼叫 check_email_exists 以查詢資料庫中是否存在指定的 email
    match users::check_email_exists(state, email).await {
        Ok(db_user) => {
            // 將查詢結果對應為 CurrentUser 類型
            Some(CurrentUser {
                email: db_user.email,
                password_hash: db_user.password,
            })
        }
        Err(_) => None, // 如果查詢失敗或使用者不存在，則傳回 None
    }
}

pub fn encode_jwt(email: String) -> Result<String, StatusCode> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("找不到 JWT_SECRET");

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
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>, StatusCode> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("找不到 JWT_SECRET");

    decode(
        &jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => StatusCode::UNAUTHORIZED,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn _hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    let hash = hash(password, DEFAULT_COST)?;
    Ok(hash)
}
