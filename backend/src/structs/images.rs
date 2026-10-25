use serde::Serialize;
use sqlx::FromRow;

/// `images` 表的一列，同時是 `GET /admin/images` 的回應型別。
/// `width` / `height` / `blur_data_url` 為模糊預覽用。上傳一定寫入寬高（欄位 nullable 只因 2026-09-30
/// 才加欄、舊列已回填）；`blur_data_url` 在小圖編碼失敗時為 None。
/// `size_bytes` 是實際落地的檔案大小（2026-10-26 加欄，舊列由 `backfill_sizes` 回填；檔案遺失的列為 None）。
#[derive(Serialize, FromRow)]
pub struct ImageRecord {
    pub id: i32,
    pub storage_key: String,
    pub url: String,
    pub status: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub blur_data_url: Option<String>,
    pub size_bytes: Option<i64>,
}

/// 單張圖的版位與模糊預覽，隨 `GET /blogs/{id}` 以 `images: { url → 此結構 }` 下發。
#[derive(Debug, Serialize, FromRow)]
pub struct ImagePlaceholder {
    pub width: i32,
    pub height: i32,
    pub blur_data_url: Option<String>,
}

/// `repositories::images::get_placeholders_by_urls` 的一列：URL + 該圖的版位資訊
#[derive(FromRow)]
pub struct UrlPlaceholder {
    pub url: String,
    #[sqlx(flatten)]
    pub placeholder: ImagePlaceholder,
}
