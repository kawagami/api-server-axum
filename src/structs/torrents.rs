use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub const STATUS_PENDING: &str = "pending";
pub const STATUS_DOWNLOADING: &str = "downloading";
pub const STATUS_COMPLETED: &str = "completed";

#[derive(Serialize, FromRow, Clone)]
pub struct Torrent {
    pub id: i32,
    pub info_hash: String,
    pub magnet_uri: String,
    pub name: Option<String>,
    pub status: String,
    pub total_size: Option<i64>,
    pub files: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// files JSONB 內的單一檔案
#[derive(Serialize, Deserialize, Clone)]
pub struct TorrentFile {
    pub index: usize,
    pub path: String,
    pub size: u64,
}

#[derive(Deserialize)]
pub struct CreateTorrent {
    pub magnet_uri: String,
}

#[derive(Serialize)]
pub struct TorrentPaginatedResponse {
    pub data: Vec<Torrent>,
    pub total: i64,
}

/// 下載連結的短效 JWT claims（與 admin/member JWT 無關）
#[derive(Serialize, Deserialize)]
pub struct TorrentDownloadClaims {
    pub exp: usize,
    pub purpose: String,
    /// 發行者 email — 下載時即時重查權限，權限被拔掉連結立即失效
    pub sub: String,
    pub torrent_id: i32,
    pub file_index: usize,
}

pub const DOWNLOAD_TOKEN_PURPOSE: &str = "torrent_download";

#[derive(Serialize)]
pub struct DownloadLink {
    pub file_index: usize,
    pub path: String,
    pub size: u64,
    pub url: String,
    pub expires_at: DateTime<Utc>,
}
