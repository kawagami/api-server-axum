use crate::{
    errors::{AppError, RequestError},
    repositories::firebase::{delete as repo_delete, images as repo_images, upload as repo_upload},
    routes::auth,
    state::AppStateV2,
    structs::firebase::{DeleteImageRequest, FirebaseImage, Image},
};
use axum::{
    extract::{Multipart, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use reqwest::multipart;

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    // 受保護的路由，包含圖片上傳與刪除功能，需通過身份驗證
    let protected_routes = Router::new().route("/", post(upload).delete(delete)).layer(
        middleware::from_fn_with_state(state.clone(), auth::authorize),
    );

    // 合併公開與受保護的路由，提供圖片查詢及管理功能
    Router::new()
        .route("/", get(images)) // 公開路由：獲取圖片列表
        .merge(protected_routes) // 受保護路由：上傳與刪除圖片
}

// 處理圖片上傳請求
pub async fn upload(
    State(state): State<AppStateV2>,
    mut multipart: Multipart, // 接收多部分請求格式的文件
) -> Result<Json<FirebaseImage>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::RequestError(RequestError::MultipartError(err.into())))?
    {
        // 取得文件名稱
        let file_name = field
            .file_name()
            .ok_or_else(|| {
                AppError::RequestError(RequestError::InvalidContent(
                    "Missing file name".to_string(),
                ))
            })?
            .to_string();

        // 取得內容類型（MIME 類型）
        let content_type = field
            .content_type()
            .ok_or_else(|| {
                AppError::RequestError(RequestError::InvalidContent(
                    "Missing content type".to_string(),
                ))
            })?
            .to_string();

        // 讀取文件內容
        let data = field
            .bytes()
            .await
            .map_err(|err| AppError::RequestError(RequestError::MultipartError(err.into())))?;

        // 構建 `multipart::Part`，用於上傳到 Firebase
        let part = multipart::Part::bytes(data.to_vec())
            .file_name(file_name.clone())
            .mime_str(&content_type)
            .map_err(|err| AppError::RequestError(RequestError::InvalidContent(err.to_string())))?;

        let form = multipart::Form::new().part("file", part);
        let res = repo_upload(&state, form).await?; // 調用 Firebase 上傳函數

        if res.status().is_success() {
            return res
                .json()
                .await
                .map(Json)
                .map_err(|err| AppError::RequestError(RequestError::InvalidJson(err.into())));
        }
    }

    Err(AppError::RequestError(RequestError::NotFound)) // 若未提供有效文件則返回錯誤
}

// 處理獲取圖片列表請求
pub async fn images(State(state): State<AppStateV2>) -> Result<Json<Vec<Image>>, AppError> {
    let images = repo_images(&state).await?; // 從 Firebase 獲取圖片列表

    Ok(Json(images)) // 返回 JSON 格式的圖片列表
}

// 處理刪除圖片請求
pub async fn delete(
    State(state): State<AppStateV2>,
    Json(delete_data): Json<DeleteImageRequest>, // 解析請求 JSON
) -> Result<Json<()>, AppError> {
    repo_delete(&state, delete_data).await?; // 調用 Firebase 刪除函數

    Ok(Json(())) // 返回成功響應
}
