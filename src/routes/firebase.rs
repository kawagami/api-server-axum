use crate::errors::UploadError;
use axum::{extract::Multipart, Json};
use reqwest::{multipart, Client};
use serde::Serialize;

#[derive(Serialize)]
pub struct Response {
    image_url: String,
}

pub async fn upload(mut multipart: Multipart) -> Result<Json<Response>, UploadError> {
    let client = Client::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| UploadError::GetNextFieldFail)?
    {
        // let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        // Create a form part with the received file
        let part = multipart::Part::bytes(data.to_vec())
            .file_name(file_name.clone())
            .mime_str(&content_type)
            .unwrap();

        // Create a multipart form
        let form = multipart::Form::new().part("file", part);

        // Send the file as a POST request to http://firebase:5000/upload
        let res = client
            .post("http://firebase:5000/upload")
            // .post("http://host.docker.internal:5000/upload")
            .multipart(form)
            .send()
            .await
            .map_err(|_| UploadError::ConnectFail)?;

        // Check the response status and body if needed
        if res.status().is_success() {
            return Ok(Json(Response {
                image_url: res.text().await.unwrap(),
            }));
        }
    }
    // Err((StatusCode::INTERNAL_SERVER_ERROR, "好像圖片上傳失敗".to_owned()))
    Err(UploadError::NotThing)
}
