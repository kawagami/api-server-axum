use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("圖片 server 連接失敗: {0}")]
    ConnectFail(#[source] anyhow::Error), // 使用通用錯誤類型
    #[error("multipart next_field await 失敗: {0}")]
    GetNextFieldFail(#[source] anyhow::Error),
    #[error("multipart next_field 沒東西")]
    NotThing,
    #[error("field bytes 時失敗: {0}")]
    ReadBytesFail(#[source] anyhow::Error),
    #[error("無效的 json 格式: {0}")]
    InvalidJson(#[source] anyhow::Error), // 更新為通用錯誤
    #[error("Invalid response from the server: {0}")]
    InvalidResponse(#[source] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 紀錄完整錯誤資訊，包括來源錯誤（如果存在）
        tracing::error!("AppError occurred: {:#?}", self);

        let status_code = match &self {
            AppError::ConnectFail(_) => StatusCode::BAD_GATEWAY,
            AppError::GetNextFieldFail(_) => StatusCode::BAD_REQUEST,
            AppError::NotThing => StatusCode::NOT_FOUND,
            AppError::ReadBytesFail(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidJson(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::InvalidResponse(_) => StatusCode::BAD_GATEWAY,
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
