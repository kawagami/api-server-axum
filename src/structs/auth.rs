use axum::{
    body::Body,
    extract::Json,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

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

impl IntoResponse for AuthError {
    fn into_response(self) -> Response<Body> {
        let body = Json(json!({
            "error": self.message,
        }));

        (self.status_code, body).into_response()
    }
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
