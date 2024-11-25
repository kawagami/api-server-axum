use crate::image_processor::resize_image;
use axum::{
    body::Body,
    extract::{Multipart, Path},
    http::header,
    response::{IntoResponse, Response},
};

// 圖片上傳處理路由
pub async fn resize(
    Path((witdh, height, format)): Path<(u32, u32, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // 預設輸出格式為 PNG
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        // 使用公共庫處理圖片
        match resize_image(&data, witdh, height, &format) {
            Ok(resized_data) => {
                // 設置正確的 Content-Type
                let content_type = match format.as_str() {
                    "png" => "image/png",
                    "webp" => "image/webp",
                    "jpeg" | "jpg" => "image/jpeg",
                    "bmp" => "image/bmp",
                    "gif" => "image/gif",
                    "ico" => "image/x-icon",
                    "tiff" => "image/tiff",
                    _ => "image/png",
                };

                // 返回圖片作為 Body
                let body = Body::from(resized_data);
                return Response::builder()
                    .header(header::CONTENT_TYPE, content_type)
                    .body(body)
                    .unwrap();
            }
            Err(_) => {
                return Response::builder()
                    .status(500)
                    .body(Body::from("Image processing failed"))
                    .unwrap();
            }
        }
    }

    // 如果沒有上傳文件，返回錯誤
    Response::builder()
        .status(400)
        .body(Body::from("No file uploaded"))
        .unwrap()
}
