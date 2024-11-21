use crate::image_processor::resize_image;
use axum::{
    body::Body,
    extract::Multipart,
    http::header,
    response::{IntoResponse, Response},
};

// 圖片上傳處理路由
pub async fn resize(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        // 使用公共庫處理圖片
        match resize_image(&data, 600, 600) {
            Ok(resized_data) => {
                // 返回圖片作為 Body
                let body = Body::from(resized_data);
                return Response::builder()
                    .header(header::CONTENT_TYPE, "image/png")
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
