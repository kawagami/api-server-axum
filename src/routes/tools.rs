use crate::image_processor::resize_image;
use crate::structs::tools::ImageFormat;
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

pub async fn resize(
    Path((width, height, format)): Path<(u32, u32, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let data = field.bytes().await.unwrap();

        // 將字串轉換為 ImageFormat
        let image_format = match ImageFormat::from_str(&format) {
            Some(format) => format,
            None => ImageFormat::PNG, // 預設使用 PNG
        };

        match resize_image(&data, width, height, &format) {
            Ok(resized_data) => {
                let body = Body::from(resized_data);
                return Response::builder()
                    .header(header::CONTENT_TYPE, image_format.content_type())
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

    Response::builder()
        .status(400)
        .body(Body::from("No file uploaded"))
        .unwrap()
}
