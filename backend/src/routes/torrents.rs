use crate::{
    errors::AppError,
    services::torrents as torrents_service,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        pagination::PageQuery,
        roles::Perm,
        torrents::{CreateTorrent, DownloadLink, Torrent, TorrentPaginatedResponse},
    },
};
use axum::{
    extract::{Extension, Path, Query, Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use tower::ServiceExt;
use tower_http::services::ServeFile;

pub fn new(state: AppState) -> Router<AppState> {
    let protected = super::with_auth(
        state,
        Router::new()
            .route("/", post(create_torrent).get(list_torrents))
            .route("/storage", get(get_storage_stats))
            .route("/{id}", get(get_torrent).delete(delete_torrent))
            .route("/{id}/pending", patch(reset_torrent_pending))
            .route("/{id}/download_links", post(create_download_links)),
    );

    // 檔案下載走短效簽名 token（瀏覽器/下載器帶不了 Authorization header），不掛 JWT middleware
    Router::new()
        .route("/{id}/files/{file_index}", get(download_file))
        .merge(protected)
}

async fn create_torrent(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(payload): Json<CreateTorrent>,
) -> Result<(StatusCode, Json<Torrent>), AppError> {
    auth_user.require_permission(Perm::TorrentCreate)?;
    let torrent =
        torrents_service::create(&state, &payload.magnet_uri, &auth_user.email, Some(auth_user.id)).await?;
    Ok((StatusCode::CREATED, Json(torrent)))
}

#[derive(Deserialize)]
struct StatusFilter {
    status: Option<String>,
}

async fn list_torrents(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(filter): Query<StatusFilter>,
    Query(page): Query<PageQuery>,
) -> Result<Json<TorrentPaginatedResponse>, AppError> {
    auth_user.require_permission(Perm::TorrentRead)?;
    let (limit, offset) = page.to_limit_offset(50);
    Ok(Json(
        crate::repositories::torrents::list(
            state.get_pool(),
            filter.status,
            auth_user.owner_filter(),
            limit,
            offset,
        )
        .await?,
    ))
}

async fn get_storage_stats(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::TorrentRead)?;
    Ok(Json(torrents_service::storage_stats(&state).await?))
}

async fn get_torrent(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    auth_user.require_permission(Perm::TorrentRead)?;
    auth_user.require_owner(crate::repositories::torrents::get_owner(state.get_pool(), id).await?)?;
    Ok(Json(torrents_service::detail(&state, id).await?))
}

async fn reset_torrent_pending(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::TorrentCreate)?;
    auth_user.require_owner(crate::repositories::torrents::get_owner(state.get_pool(), id).await?)?;
    torrents_service::reset_pending(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_torrent(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    auth_user.require_permission(Perm::TorrentDelete)?;
    auth_user.require_owner(crate::repositories::torrents::get_owner(state.get_pool(), id).await?)?;
    torrents_service::delete(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_download_links(
    Extension(auth_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, Json<Vec<DownloadLink>>), AppError> {
    auth_user.require_permission(Perm::TorrentRead)?;
    auth_user.require_owner(crate::repositories::torrents::get_owner(state.get_pool(), id).await?)?;
    let links = torrents_service::create_download_links(&state, id, &auth_user.email).await?;
    Ok((StatusCode::CREATED, Json(links)))
}

#[derive(Deserialize)]
struct DownloadQuery {
    token: String,
}

/// 簽名 URL 下載：token 驗證後交給 ServeFile（內建 Range / ETag，支援續傳）
async fn download_file(
    State(state): State<AppState>,
    Path((id, file_index)): Path<(i32, usize)>,
    Query(query): Query<DownloadQuery>,
    request: Request,
) -> Result<Response, AppError> {
    let (path, filename) =
        torrents_service::resolve_download_file(&state, id, file_index, &query.token).await?;

    let mut response = ServeFile::new(&path)
        .oneshot(request)
        .await
        .map_err(|e| crate::errors::SystemError::Internal(format!("serve file failed: {e}")))?
        .into_response();

    // RFC 5987 percent-encoding，檔名含中文/空白也安全
    let encoded: String = form_urlencoded::byte_serialize(filename.as_bytes()).collect();
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename*=UTF-8''{encoded}")) {
        response.headers_mut().insert(header::CONTENT_DISPOSITION, value);
    }

    Ok(response)
}
