use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum UploadError {
    ConnectFail,
    GetNextFieldFail,
    NotThing,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            UploadError::ConnectFail => (StatusCode::INTERNAL_SERVER_ERROR, "圖片 server 連接失敗"),
            UploadError::GetNextFieldFail => (StatusCode::BAD_REQUEST, "multipart next_field await 失敗"),
            UploadError::NotThing => (StatusCode::BAD_REQUEST, "multipart next_field 沒東西"),
        };
        (status, error_message).into_response()
    }
}
