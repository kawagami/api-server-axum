use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    ConnectFail,
    GetNextFieldFail,
    NotThing,
    ReadBytesFail,
    InvalidJson,
    GetDbDataFail,
    DbInsertFail,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ConnectFail => (StatusCode::INTERNAL_SERVER_ERROR, "圖片 server 連接失敗"),
            AppError::GetNextFieldFail => {
                (StatusCode::BAD_REQUEST, "multipart next_field await 失敗")
            }
            AppError::NotThing => (StatusCode::BAD_REQUEST, "multipart next_field 沒東西"),
            AppError::ReadBytesFail => (StatusCode::BAD_REQUEST, "field bytes 時失敗"),
            AppError::InvalidJson => (StatusCode::BAD_REQUEST, "無效的 json 格式"),
            AppError::GetDbDataFail => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "取得 firebase_images 失敗",
            ),
            AppError::DbInsertFail => (StatusCode::INTERNAL_SERVER_ERROR, "DB INSERT FAIL"),
        };
        (status, error_message).into_response()
    }
}

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
