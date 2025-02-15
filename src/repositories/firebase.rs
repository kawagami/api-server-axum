use crate::{
    errors::{AppError, RequestError, SystemError},
    state::AppStateV2,
    structs::firebase::{ApiResponse, DeleteImageRequest, Image},
};
use reqwest::{multipart::Form, Response};

/// 上傳圖片到 FastAPI 服務器
pub async fn upload(state: &AppStateV2, form: Form) -> Result<Response, AppError> {
    let client = state.get_http_client();
    let url = format!("{}{}", state.get_fastapi_upload_host(), "/upload-image");

    client
        .post(url)
        .multipart(form)
        .send()
        .await
        .map_err(|err| AppError::ConnectionError(err.into()))
}

/// 獲取圖片列表
pub async fn images(state: &AppStateV2) -> Result<Vec<Image>, AppError> {
    let client = state.get_http_client();
    let url = format!("{}{}", state.get_fastapi_upload_host(), "/list-images");

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|err| AppError::ConnectionError(err.into()))?;

    // 先檢查狀態碼
    if !response.status().is_success() {
        return Err(AppError::RequestError(RequestError::InvalidContent(
            format!("Server returned status code: {}", response.status()),
        )));
    }

    let api_response = response
        .json::<ApiResponse>()
        .await
        .map_err(|err| AppError::RequestError(RequestError::InvalidJson(err.into())))?;

    // 檢查回應是否包含檔案列表
    match api_response.files {
        files if files.is_empty() => Ok(vec![]),
        files => Ok(files),
    }
}

/// 刪除指定圖片
pub async fn delete(state: &AppStateV2, delete_data: DeleteImageRequest) -> Result<(), AppError> {
    let client = state.get_http_client();
    let url = format!("{}{}", state.get_fastapi_upload_host(), "/delete-image");

    let response = client
        .delete(url)
        .json(&delete_data)
        .send()
        .await
        .map_err(|err| AppError::ConnectionError(err.into()))?;

    // 檢查刪除操作是否成功
    if !response.status().is_success() {
        return Err(AppError::SystemError(SystemError::Internal(format!(
            "Failed to delete image. Status: {}",
            response.status()
        ))));
    }

    Ok(())
}
