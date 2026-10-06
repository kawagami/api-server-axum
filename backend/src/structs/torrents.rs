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

/// 進行中任務的即時進度（不存 DB）。詳情 API 的 `live` 與 WS `torrent_progress` 共用這個形狀
/// —— 前端 `types/torrent.ts` 的 `TorrentLive` / `TorrentProgressEvent extends TorrentLive`。
#[derive(Serialize)]
pub struct TorrentLive {
    /// 0–100，四捨五入到小數兩位
    pub progress: f64,
    pub progress_bytes: u64,
    pub total_bytes: u64,
    /// librqbit 的人類可讀速率（如 `"1.23 MiB/s"`），不是數字
    pub down_speed: String,
    pub peers: u32,
}

/// `GET /admin/torrents/{id}`：DB row 攤平 + 進行中任務的 `live`（非進行中則不帶此欄）
#[derive(Serialize)]
pub struct TorrentDetail {
    #[serde(flatten)]
    pub torrent: Torrent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<TorrentLive>,
}

/// WS `torrent_progress` 的 payload
#[derive(Serialize)]
pub struct TorrentProgressEvent<'a> {
    pub id: i32,
    pub name: &'a str,
    #[serde(flatten)]
    pub live: TorrentLive,
}

/// `GET /admin/torrents/storage`
#[derive(Serialize)]
pub struct TorrentStorage {
    pub disk: DiskUsage,
    pub torrent: TorrentQuota,
}

/// TORRENT_PATH 所在檔案系統（statvfs）
#[derive(Serialize)]
pub struct DiskUsage {
    pub total_bytes: u64,
    /// 非 root 可用容量
    pub available_bytes: u64,
}

/// torrent 配額：DB 內所有任務的 total_size 加總 vs `torrent_max_total_size_gb`
#[derive(Serialize)]
pub struct TorrentQuota {
    pub used_bytes: i64,
    pub max_bytes: i64,
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


/// 下載連結的短效 JWT claims（與 admin/member JWT 無關）
#[derive(Serialize, Deserialize)]
pub struct TorrentDownloadClaims {
    pub exp: usize,
    pub purpose: String,
    /// 發行者 user id — 下載時即時重查權限，權限被拔掉連結立即失效
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
