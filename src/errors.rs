use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/*
要新增對應新的錯誤類型的話要完成以下三項
1. 在 enum AppError 中增加 名稱(類型)
2. 在 match 中 return 對應的 (status, Json(ErrorJson{MESSAGE}))
3. 為 AppError 實現該錯誤類型的 From trait
*/
// 1
pub enum AppError {
    SqlxInputError(sqlx::error::Error),
    SqlxError(sqlx::error::Error),
    Undefine(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorJson {
            message: String,
        }

        // 2
        let (status, message) = match self {
            AppError::SqlxError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorJson {
                    message: err.to_string(),
                }),
            ),
            AppError::Undefine(err) => (
                StatusCode::IM_A_TEAPOT,
                Json(ErrorJson {
                    message: err.to_string(),
                }),
            ),
            AppError::SqlxInputError(err) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorJson {
                    message: err.to_string(),
                }),
            ),
        };

        (status, message).into_response()
    }
}

// 3
impl From<sqlx::error::Error> for AppError {
    fn from(rejection: sqlx::error::Error) -> Self {
        Self::SqlxError(rejection)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::Undefine(error)
    }
}
