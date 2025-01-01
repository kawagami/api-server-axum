use crate::image_processor::resize_image;
use crate::{state::AppStateV2, structs::tools::Params};
use axum::{
    body::Body,
    extract::{Multipart, Path, Query},
    http::header,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rand::{distributions::Alphanumeric, Rng};

pub fn new() -> Router<AppStateV2> {
    Router::new()
        .route("/new_password", get(new_password))
        .route("/image/{width}/{height}/{format}/resize", post(resize))
}

pub async fn new_password(Query(params): Query<Params>) -> Result<Json<Vec<String>>, ()> {
    // 預設長度為 8，預設數量為 1
    let len = params.length;
    let cnt = params.count;

    let mut rng = rand::thread_rng();

    // 生成指定數量的隨機字串
    let result = (0..cnt)
        .map(|_| (0..len).map(|_| rng.sample(Alphanumeric) as char).collect())
        .collect();

    Ok(Json(result))
}

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
