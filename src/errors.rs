use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("圖片 server 連接失敗: {0}")]
    ConnectFail(#[source] anyhow::Error),
    #[error("multipart next_field await 失敗: {0}")]
    GetNextFieldFail(#[source] anyhow::Error),
    #[error("multipart next_field 沒東西")]
    NotThing,
    #[error("field bytes 時失敗: {0}")]
    ReadBytesFail(#[source] anyhow::Error),
    #[error("無效的 json 格式: {0}")]
    InvalidJson(#[source] anyhow::Error),
    #[error("Invalid response from the server: {0}")]
    InvalidResponse(#[source] anyhow::Error),

    // 認證錯誤
    #[error("請在 Header 中提供 JWT token")]
    MissingToken,
    #[error("Header 格式無效")]
    InvalidHeaderFormat,
    #[error("Token 無法解碼")]
    DecodeTokenFail,
    #[error("使用者未被授權")]
    UnauthorizedUser,

    // 認證相關錯誤
    #[error("無法找到指定的使用者")]
    UserNotFound,
    #[error("密碼驗證失敗")]
    PasswordVerificationFailed,
    #[error("JWT 簽名生成失敗")]
    JwtEncodeFailed,
    #[error("JWT 解碼失敗")]
    JwtDecodeFailed,
    #[error("JWT 已過期")]
    JwtExpired,

    // Redis 或系統錯誤
    #[error("Redis 操作失敗: {0}")]
    RedisError(String),
    #[error("環境變數缺失: {0}")]
    MissingEnvVariable(String),
    #[error("內部伺服器錯誤: {0}")]
    InternalError(String),
}

impl AppError {
    /// 將每種錯誤與對應的 HTTP 狀態碼綁定
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ConnectFail(_) => StatusCode::BAD_GATEWAY,
            AppError::GetNextFieldFail(_) => StatusCode::BAD_REQUEST,
            AppError::NotThing => StatusCode::NOT_FOUND,
            AppError::ReadBytesFail(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidJson(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::InvalidResponse(_) => StatusCode::BAD_GATEWAY,

            // 認證錯誤對應的 HTTP 狀態碼
            AppError::MissingToken => StatusCode::FORBIDDEN,
            AppError::InvalidHeaderFormat => StatusCode::FORBIDDEN,
            AppError::DecodeTokenFail => StatusCode::UNAUTHORIZED,
            AppError::UnauthorizedUser => StatusCode::UNAUTHORIZED,
            AppError::UserNotFound => StatusCode::UNAUTHORIZED,
            AppError::PasswordVerificationFailed => StatusCode::UNAUTHORIZED,
            AppError::JwtEncodeFailed => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtDecodeFailed => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JwtExpired => StatusCode::UNAUTHORIZED,
            AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MissingEnvVariable(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("AppError occurred: {:#?}", self);
        let status_code = self.status_code();
        let error_message = self.to_string();
        (status_code, error_message).into_response()
    }
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
