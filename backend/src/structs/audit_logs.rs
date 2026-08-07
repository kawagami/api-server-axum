use chrono::{DateTime, Utc};
use serde::Deserialize;

/// `GET /admin/audit_logs` 的篩選參數。
///
/// 放在 structs/ 而非 route 內是為了讓 repository 的 list 與 count 共用同一份
/// 篩選條件（`repositories/audit_logs.rs` 的 `AUDIT_FILTER`），不必把 5 個參數平鋪兩次
/// —— 兩邊 WHERE 一旦漂移，`total` 就會與實際筆數對不上（範本同 `structs/logs.rs`）。
#[derive(Deserialize, Default)]
pub struct AuditLogQuery {
    pub user_email: Option<String>,
    pub method: Option<String>,
    /// 路徑模糊比對
    pub path: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}
