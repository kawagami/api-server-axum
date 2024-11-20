use axum::{
    body::Body,
    extract::Multipart,
    http::header,
    response::{IntoResponse, Response},
};
use image::ImageFormat;
use std::io::Cursor;

// 圖片上傳處理路由
pub async fn resize(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        // 將圖片加載為 DynamicImage
        let img = image::load_from_memory(&data).unwrap();

        // 調整圖片大小
        let resized_img = img.resize(200, 200, image::imageops::FilterType::Triangle);

        // 將處理後的圖片寫入內存緩衝區
        let mut buffer = Cursor::new(Vec::new());
        resized_img
            .write_to(&mut buffer, ImageFormat::Png)
            .unwrap();
        let resized_data = buffer.into_inner();

        // 返回圖片作為 Body
        let body = Body::from(resized_data);
        return Response::builder()
            .header(header::CONTENT_TYPE, "image/png")
            .body(body)
            .unwrap();
    }

    // 如果沒有上傳文件，返回錯誤
    Response::builder()
        .status(400)
        .body(Body::from("No file uploaded"))
        .unwrap()
}
