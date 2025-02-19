use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

// 用於 API 回應的錯誤結構
#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

#[derive(Error, Debug)]
pub enum AppError {
    // HTTP 相關錯誤
    #[error("服務連接失敗: {0}")]
    ConnectionError(#[source] anyhow::Error),

    // 請求處理錯誤
    #[error("請求處理失敗: {0}")]
    RequestError(#[from] RequestError),

    // 認證相關錯誤
    #[error("認證失敗: {0}")]
    AuthError(#[from] AuthError),

    // 系統錯誤
    #[error("系統錯誤: {0}")]
    SystemError(#[from] SystemError),

    #[error("WebSocket錯誤: {0}")]
    WebSocketError(#[from] WebSocketError),
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("multipart 欄位處理失敗: {0}")]
    MultipartError(#[source] anyhow::Error),

    #[error("無效的請求內容: {0}")]
    InvalidContent(String),

    #[error("無效的 JSON 格式: {0}")]
    InvalidJson(#[source] anyhow::Error),

    #[error("找不到資源")]
    NotFound,
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("缺少認證 Token")]
    MissingToken,

    #[error("無效的認證標頭")]
    InvalidHeader,

    #[error("Token 已過期")]
    TokenExpired,

    #[error("無效的 Token")]
    InvalidToken,

    #[error("未授權的存取")]
    Unauthorized,

    #[error("使用者不存在")]
    UserNotFound,

    #[error("密碼驗證失敗")]
    InvalidPassword,
}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("Redis 錯誤: {0}")]
    RedisError(String),

    #[error("環境變數缺失: {0}")]
    EnvVarMissing(String),

    #[error("內部錯誤: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("無效的Token")]
    InvalidToken,

    #[error("WebSocket連接失敗: {0}")]
    ConnectionFailed(String),

    #[error("訊息廣播失敗: {0}")]
    BroadcastFailed(String),

    #[error("在線用戶管理失敗: {0}")]
    UserManagementFailed(String),

    #[error("訊息解析失敗: {0}")]
    MessageDecodeFailed(String),
}

impl AppError {
    fn error_response(&self) -> ErrorResponse {
        let status = self.status_code();

        ErrorResponse {
            code: status.as_u16(),
            message: self.to_string(),
            details: self.error_details(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConnectionError(_) => StatusCode::BAD_GATEWAY,
            Self::RequestError(err) => match err {
                RequestError::MultipartError(_) => StatusCode::BAD_REQUEST,
                RequestError::InvalidContent(_) => StatusCode::BAD_REQUEST,
                RequestError::InvalidJson(_) => StatusCode::UNPROCESSABLE_ENTITY,
                RequestError::NotFound => StatusCode::NOT_FOUND,
            },
            Self::AuthError(err) => match err {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidHeader => StatusCode::BAD_REQUEST,
                AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                AuthError::Unauthorized => StatusCode::FORBIDDEN,
                AuthError::UserNotFound => StatusCode::NOT_FOUND,
                AuthError::InvalidPassword => StatusCode::UNAUTHORIZED,
            },
            Self::SystemError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebSocketError(err) => match err {
                WebSocketError::InvalidToken => StatusCode::UNAUTHORIZED,
                WebSocketError::ConnectionFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
                WebSocketError::BroadcastFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
                WebSocketError::UserManagementFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
                WebSocketError::MessageDecodeFailed(_) => StatusCode::BAD_REQUEST,
            },
        }
    }

    fn error_details(&self) -> Option<String> {
        if cfg!(debug_assertions) {
            Some(format!("{:#?}", self))
        } else {
            None
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_response = self.error_response();

        // Log the error with different levels based on severity
        match &self {
            AppError::SystemError(_) => {
                tracing::error!(?self, "System error occurred");
            }
            AppError::ConnectionError(_) => {
                tracing::error!(?self, "Connection error occurred");
            }
            AppError::AuthError(_) => {
                tracing::warn!(?self, "Authentication error occurred");
            }
            AppError::RequestError(_) => {
                tracing::debug!(?self, "Request error occurred");
            }
            AppError::WebSocketError(_) => {
                tracing::debug!(?self, "WS error occurred");
            }
        }

        (self.status_code(), Json(error_response)).into_response()
    }
}

// 便利函數
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self::SystemError(SystemError::Internal(err.to_string()))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::SystemError(SystemError::Internal(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_format() {
        let error = AppError::AuthError(AuthError::MissingToken);
        let response = error.error_response();

        assert_eq!(response.code, 401);
        assert_eq!(response.message, "認證失敗: 缺少認證 Token");
    }

    #[test]
    fn test_status_codes() {
        let auth_error = AppError::AuthError(AuthError::MissingToken);
        assert_eq!(auth_error.status_code(), StatusCode::UNAUTHORIZED);

        let system_error = AppError::SystemError(SystemError::Internal("test".to_string()));
        assert_eq!(
            system_error.status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
