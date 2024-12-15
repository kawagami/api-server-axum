use crate::{
    state::AppStateV2,
    structs::auth::{AuthError, Claims, CurrentUser, SignInData},
};
use axum::{
    body::Body,
    extract::{Json, Request, State},
    http,
    http::{Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde_json::json;

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn _hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    let hash = hash(password, DEFAULT_COST)?;
    Ok(hash)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response<Body> {
        let body = Json(json!({
            "error": self.message,
        }));

        (self.status_code, body).into_response()
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

pub async fn authorize(
    State(state): State<AppStateV2>,
    mut req: Request,
    next: Next,
) -> Result<Response<Body>, AuthError> {
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

    let token_data = match decode_jwt(token.unwrap().to_string()) {
        Ok(data) => data,
        Err(_) => {
            return Err(AuthError {
                message: "Unable to decode token".to_string(),
                status_code: StatusCode::UNAUTHORIZED,
            })
        }
    };

    // Fetch the user details from the database
    let current_user = match retrieve_user_by_email(state, &token_data.claims.email).await {
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
    // 1. Retrieve user from the database
    let user = match retrieve_user_by_email(state, &user_data.email).await {
        Some(user) => user,
        None => return Err(StatusCode::UNAUTHORIZED), // User not found
    };

    // 2. Compare the password
    if !verify_password(&user_data.password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // Handle bcrypt errors
    {
        return Err(StatusCode::UNAUTHORIZED); // Wrong password
    }

    // 3. Generate JWT
    let token = encode_jwt(user.email).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Return the token
    Ok(Json(token))
}

async fn retrieve_user_by_email(state: AppStateV2, email: &str) -> Option<CurrentUser> {
    // 呼叫 check_email_exists 以查詢資料庫中是否存在指定的 email
    match state.check_email_exists(email).await {
        Ok(db_user) => {
            // 將查詢結果對應為 CurrentUser 類型
            Some(CurrentUser {
                email: db_user.email,
                password_hash: db_user.password,
            })
        }
        Err(_) => {
            // 如果查詢失敗或使用者不存在，則傳回 None
            None
        }
    }
}
