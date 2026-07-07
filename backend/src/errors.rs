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
    // 對應該次請求的追蹤 id（同時放在 x-request-id header），使用者回報錯誤時可據此撈 log
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

// 變體統一 *Error 後綴是本專案慣例，比去掉後綴的裸名（Connection / Request…）可讀
#[allow(clippy::enum_variant_names)]
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
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("multipart 欄位處理失敗: {0}")]
    MultipartError(#[source] anyhow::Error),

    #[error("無效的請求內容: {0}")]
    InvalidContent(String),

    #[error("驗證失敗: {0}")]
    UnprocessableContent(String),

    #[error("資源衝突: {0}")]
    Conflict(String),

    #[error("儲存空間不足: {0}")]
    InsufficientStorage(String),

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

    #[error("權限不足")]
    Forbidden,

    #[error("使用者不存在")]
    UserNotFound,

    #[error("密碼驗證失敗")]
    InvalidPassword,

    // 登入失敗統一回此訊息，不區分帳號不存在/密碼錯誤，防帳號枚舉
    #[error("帳號或密碼錯誤")]
    InvalidCredentials,
}

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("內部錯誤: {0}")]
    Internal(String),

    #[error("資料庫錯誤")]
    Database(#[source] sqlx::Error),

    #[error("Redis 錯誤")]
    Redis(#[source] redis::RedisError),

    #[error("JSON 處理錯誤")]
    Json(#[source] serde_json::Error),

    #[error("時間解析錯誤")]
    TimeParse(#[source] chrono::ParseError),
}

impl AppError {
    fn error_response(&self) -> ErrorResponse {
        let status = self.status_code();

        ErrorResponse {
            code: status.as_u16(),
            message: self.to_string(),
            details: self.error_details(),
            request_id: crate::middleware::request_id::current_request_id(),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::ConnectionError(_) => StatusCode::BAD_GATEWAY,
            Self::RequestError(err) => match err {
                RequestError::MultipartError(_) => StatusCode::BAD_REQUEST,
                RequestError::InvalidContent(_) => StatusCode::BAD_REQUEST,
                RequestError::UnprocessableContent(_) => StatusCode::UNPROCESSABLE_ENTITY,
                RequestError::Conflict(_) => StatusCode::CONFLICT,
                RequestError::InsufficientStorage(_) => StatusCode::INSUFFICIENT_STORAGE,
                RequestError::NotFound => StatusCode::NOT_FOUND,
            },
            Self::AuthError(err) => match err {
                AuthError::MissingToken => StatusCode::UNAUTHORIZED,
                AuthError::InvalidHeader => StatusCode::BAD_REQUEST,
                AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
                AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
                AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
                AuthError::Forbidden => StatusCode::FORBIDDEN,
                AuthError::UserNotFound => StatusCode::UNAUTHORIZED,
                AuthError::InvalidPassword => StatusCode::UNAUTHORIZED,
                AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            },
            Self::SystemError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
        match err {
            sqlx::Error::RowNotFound => RequestError::NotFound.into(),
            e => Self::SystemError(SystemError::Database(e)),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::SystemError(SystemError::Json(err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() || err.is_connect() {
            Self::ConnectionError(err.into())
        } else if err.is_decode() {
            RequestError::InvalidContent(err.to_string()).into()
        } else {
            Self::SystemError(SystemError::Internal(err.to_string()))
        }
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        Self::SystemError(SystemError::TimeParse(err))
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        Self::SystemError(SystemError::Redis(err))
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
