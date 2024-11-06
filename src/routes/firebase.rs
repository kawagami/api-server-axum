use crate::errors::UploadError;
use axum::{extract::Multipart, Json};
use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
        let data = field
            .bytes()
            .await
            .map_err(|_| UploadError::ReadBytesFail)?;

        // Create a form part with the received file
        let part = multipart::Part::bytes(data.to_vec())
            .file_name(file_name.clone())
            .mime_str(&content_type)
            .unwrap();

        // Create a multipart form
        let form = multipart::Form::new().part("file", part);

        let res = client
            .post("http://fastapi-upload:8000/upload-image")
            // .post("http://host.docker.internal:8000/upload-image")
            .multipart(form)
            .send()
            .await
            .map_err(|_| UploadError::ConnectFail)?;

        // Check the response status and body if needed
        if res.status().is_success() {
            let upload_response = res
                .json::<Response>()
                .await
                .map_err(|_| UploadError::InvalidJson)?;
            return Ok(Json(Response {
                image_url: upload_response.image_url,
            }));
        }
    }

    Err(UploadError::NotThing)
}
