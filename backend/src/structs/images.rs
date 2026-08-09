use serde::Serialize;

/// `images` 表的一列，同時是 `GET /admin/images` 的回應型別。
#[derive(Serialize)]
pub struct ImageRecord {
    pub id: i32,
    pub storage_key: String,
    pub url: String,
    pub status: String,
}
