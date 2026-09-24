use crate::{
    errors::{AppError, RequestError, SystemError},
    repositories::torrents as torrents_repo,
    state::AppState,
    structs::{
        auth::AuthenticatedUser,
        torrents::{
            DownloadLink, TorrentDownloadClaims, TorrentFile, DOWNLOAD_TOKEN_PURPOSE,
            STATUS_COMPLETED,
        },
    },
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::{path::PathBuf, time::Duration};
use super::manager::{DEFAULT_LINK_TTL_MINUTES, setting};
use super::tasks::ensure_owner;

/// 產生所有檔案的短效簽名下載連結
pub async fn create_download_links(
    state: &AppState,
    actor: &AuthenticatedUser,
    id: i32,
) -> Result<Vec<DownloadLink>, AppError> {
    ensure_owner(state, actor, id).await?;
    let issuer_id = actor.id;
    let torrent = torrents_repo::get_by_id(state.get_pool(), id).await?;
    if torrent.status != STATUS_COMPLETED {
        return Err(RequestError::Conflict("任務尚未完成，無法下載".to_string()).into());
    }
    let files: Vec<TorrentFile> = torrent
        .files
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();

    let ttl_minutes = setting(state, "torrent_link_ttl_minutes", DEFAULT_LINK_TTL_MINUTES).max(1);
    let expires_at = Utc::now() + Duration::from_secs(ttl_minutes as u64 * 60);
    let secret = &state.get_config().jwt_secret;

    files
        .into_iter()
        .map(|f| {
            let claims = TorrentDownloadClaims {
                exp: expires_at.timestamp() as usize,
                purpose: DOWNLOAD_TOKEN_PURPOSE.to_string(),
                sub: issuer_id.to_string(),
                torrent_id: id,
                file_index: f.index,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret.as_ref()),
            )
            .map_err(|e| SystemError::Internal(format!("簽發下載 token 失敗: {e}")))?;
            Ok(DownloadLink {
                url: format!("/admin/torrents/{id}/files/{}?token={token}", f.index),
                file_index: f.index,
                path: f.path,
                size: f.size,
                expires_at,
            })
        })
        .collect()
}

/// 驗證下載 token 並解析出實體檔案路徑（含 path traversal 防護）
pub async fn resolve_download_file(
    state: &AppState,
    id: i32,
    file_index: usize,
    token: &str,
) -> Result<(PathBuf, String), AppError> {
    let claims = decode::<TorrentDownloadClaims>(
        token,
        &DecodingKey::from_secret(state.get_config().jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::AuthError(crate::errors::AuthError::InvalidToken))?
    .claims;

    if claims.purpose != DOWNLOAD_TOKEN_PURPOSE
        || claims.torrent_id != id
        || claims.file_index != file_index
    {
        return Err(AppError::AuthError(crate::errors::AuthError::InvalidToken));
    }

    // 發行者 id（token sub）
    let issuer_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::AuthError(crate::errors::AuthError::InvalidToken))?;

    // 即時重查發行者身分（同 auth middleware 的載入點）— 權限被拔掉或帳號被刪，
    // 已發出的連結立即失效
    let identity =
        crate::services::auth::load_identity(state.get_pool(), state.get_redis_pool(), issuer_id)
            .await?
            .ok_or(AppError::AuthError(crate::errors::AuthError::Forbidden))?;
    if !identity
        .permissions
        .iter()
        .any(|p| p == crate::structs::roles::Perm::TorrentRead.as_str())
    {
        return Err(AppError::AuthError(crate::errors::AuthError::Forbidden));
    }

    let torrent = torrents_repo::get_by_id(state.get_pool(), id).await?;
    if torrent.status != STATUS_COMPLETED {
        return Err(RequestError::Conflict("任務尚未完成，無法下載".to_string()).into());
    }
    let files: Vec<TorrentFile> = torrent
        .files
        .map(serde_json::from_value)
        .transpose()?
        .unwrap_or_default();
    let file = files
        .into_iter()
        .find(|f| f.index == file_index)
        .ok_or(RequestError::NotFound)?;

    let manager = state.get_torrents();
    let dir = manager.output_dir(&torrent.info_hash);
    let path = dir.join(&file.path);

    // canonicalize 後確認還在任務目錄底下，擋 metadata 帶 ../ 的惡意路徑
    let canonical = tokio::fs::canonicalize(&path)
        .await
        .map_err(|_| AppError::from(RequestError::NotFound))?;
    let canonical_dir = tokio::fs::canonicalize(&dir)
        .await
        .map_err(|_| AppError::from(RequestError::NotFound))?;
    if !canonical.starts_with(&canonical_dir) {
        return Err(RequestError::NotFound.into());
    }

    let filename = std::path::Path::new(&file.path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("torrent-{id}-{file_index}"));

    Ok((canonical, filename))
}
