use crate::{
    errors::AppError,
    state::AppStateV2,
    structs::firebase::{ApiResponse, DeleteImageRequest, Image},
};
use reqwest::{multipart::Form, Response};

pub async fn upload(state: &AppStateV2, form: Form) -> Result<Response, AppError> {
    let client = state.get_http_client();

    let url = format!("{}{}", state.get_fastapi_upload_host(), "/upload-image");

    client
        .post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|err| AppError::ConnectFail(err.into()))
}

pub async fn images(state: &AppStateV2) -> Result<Vec<Image>, AppError> {
    let client = state.get_http_client();

    let url = format!("{}{}", state.get_fastapi_upload_host(), "/list-images");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| AppError::ConnectFail(err.into()))?
        .json::<ApiResponse>()
        .await
        .map_err(|err| AppError::InvalidResponse(err.into()))?;

    Ok(response.files)
}

pub async fn delete(state: &AppStateV2, delete_data: DeleteImageRequest) -> Result<(), AppError> {
    let client = state.get_http_client();

    let url = format!("{}{}", state.get_fastapi_upload_host(), "/delete-image");

    let _ = client
        .delete(url)
        .json(&delete_data)
        .send()
        .await
        .map_err(|err| AppError::ConnectFail(err.into()))?;

    Ok(())
}
