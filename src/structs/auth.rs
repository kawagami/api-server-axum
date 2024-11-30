use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub email: String,
}

pub struct AuthError {
    pub message: String,
    pub status_code: StatusCode,
}

#[derive(Clone)]
pub struct CurrentUser {
    pub email: String,
    pub password_hash: String,
}

#[derive(Deserialize)]
pub struct SignInData {
    pub email: String,
    pub password: String,
}
