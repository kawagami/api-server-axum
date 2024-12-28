use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("圖片 server 連接失敗")]
    ConnectFail,
    #[error("multipart next_field await 失敗")]
    GetNextFieldFail,
    #[error("multipart next_field 沒東西")]
    NotThing,
    #[error("field bytes 時失敗")]
    ReadBytesFail,
    #[error("無效的 json 格式")]
    InvalidJson,
    #[error("Invalid response from the server")]
    InvalidResponse,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            AppError::ConnectFail => StatusCode::BAD_GATEWAY,
            AppError::GetNextFieldFail => StatusCode::BAD_REQUEST,
            AppError::NotThing => StatusCode::NOT_FOUND,
            AppError::ReadBytesFail => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidJson => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::InvalidResponse => StatusCode::BAD_GATEWAY,
        };

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
