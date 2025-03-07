use crate::image_processor::resize_image;
use crate::structs::tools::ImageFormat;
use crate::{state::AppStateV2, structs::tools::Params};
use axum::{
    body::Body,
    extract::{Multipart, Path, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rand::{distributions::Alphanumeric, Rng};
use std::str::FromStr;

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/new_password", get(new_password))
        .route("/image/{width}/{height}/{format}/resize", post(resize))
}

// 自訂錯誤類型
#[derive(Debug)]
pub enum AppError {
    ValidationError(String),
    ProcessingError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::ProcessingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        Response::builder()
            .status(status)
            .body(Body::from(message))
            .unwrap()
    }
}

pub async fn new_password(Query(params): Query<Params>) -> Result<Json<Vec<String>>, AppError> {
    let mut rng = rand::thread_rng();

    // 生成指定數量的隨機字串
    let result = (0..params.count)
        .map(|_| {
            (0..params.length)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect()
        })
        .collect();

    Ok(Json(result))
}

pub async fn resize(
    Path((width, height, format)): Path<(u32, u32, String)>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    // 基本參數驗證
    if width == 0 || height == 0 {
        return Err(AppError::ValidationError(
            "Width and height must be greater than 0".to_string(),
        ));
    }

    // 解析圖片格式
    let image_format = ImageFormat::from_str(&format).unwrap_or_default();

    // 處理上傳的檔案
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::ProcessingError(format!("Failed to process multipart form: {}", e))
    })? {
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::ProcessingError(format!("Failed to read file data: {}", e)))?;

        // 處理圖片
        let resized_data = resize_image(&data, width, height, &format)
            .map_err(|_| AppError::ProcessingError("Failed to process image".to_string()))?;

        // 回傳處理後的圖片
        let response = Response::builder()
            .header(header::CONTENT_TYPE, image_format.content_type())
            .body(Body::from(resized_data))
            .map_err(|e| AppError::ProcessingError(format!("Failed to create response: {}", e)))?;

        return Ok(response);
    }

    Err(AppError::ValidationError("No file uploaded".to_string()))
}
