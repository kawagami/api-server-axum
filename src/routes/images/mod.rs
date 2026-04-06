use crate::{repositories::images as images_repo, routes::auth, state::AppStateV2};
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

pub fn new(state: AppStateV2) -> Router<AppStateV2> {
    Router::new()
        .route("/", get(get_images))
        .route("/upload", post(upload_image))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::authorize,
        ))
}

async fn get_images(State(state): State<AppStateV2>) -> impl IntoResponse {
    match images_repo::get_all_images(&state).await {
        Ok(images) => Json(images).into_response(),
        Err(e) => {
            tracing::error!("get images failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response()
        }
    }
}

async fn upload_image(
    State(state): State<AppStateV2>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let content_type = field.content_type().unwrap_or("image/jpeg").to_string();

        // field 本身就是 stream，直接傳入，不先 .bytes()
        let (storage_key, url) = match state.get_storage().upload(field, &content_type).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("storage upload failed: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "upload failed").into_response();
            }
        };

        match images_repo::insert_image(&state, &storage_key, &url).await {
            Ok(record) => {
                return (
                    StatusCode::CREATED,
                    Json(serde_json::json!({ "id": record.id, "url": record.url })),
                )
                    .into_response()
            }
            Err(e) => {
                tracing::error!("db insert failed: {:?}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
            }
        }
    }

    (StatusCode::BAD_REQUEST, "no file provided").into_response()
}
