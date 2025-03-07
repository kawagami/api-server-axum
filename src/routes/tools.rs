use crate::errors::{AppError, RequestError, SystemError};
use crate::image_processor::resize_image;
use crate::structs::tools::ImageFormat;
use crate::{state::AppStateV2, structs::tools::Params};
use axum::{
    body::Body,
    extract::{Multipart, Path, Query},
    http::header,
    response::Response,
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
        return Err(RequestError::InvalidJson(anyhow::anyhow!("test InvalidJson")).into());
    }

    // 解析圖片格式
    let image_format = ImageFormat::from_str(&format).unwrap_or_default();

    // 處理上傳的檔案
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| RequestError::MultipartError(e.into()))?
    {
        let data = field
            .bytes()
            .await
            .map_err(|e| SystemError::Internal(e.to_string()))?;

        // 處理圖片
        let resized_data = resize_image(&data, width, height, &format)
            .map_err(|e| SystemError::Internal(e.to_string()))?;

        // 回傳處理後的圖片
        let response = Response::builder()
            .header(header::CONTENT_TYPE, image_format.content_type())
            .body(Body::from(resized_data))
            .map_err(|e| SystemError::Internal(e.to_string()))?;

        return Ok(response);
    }

    Err(SystemError::Internal("預期外錯誤".to_string()).into())
}
