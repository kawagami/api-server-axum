use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// `admin_audit_logs` 表的一列，同時是 `GET /admin/audit_logs` 的回應型別。
#[derive(Serialize, sqlx::FromRow)]
pub struct AuditLog {
    pub id: i64,
    /// `admin` / `member` —— 兩種身分的 `user_email` 語意不同，見 `AuditEntry`
    pub actor_type: String,
    pub user_email: String,
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub status_code: i16,
    /// 對應 `logs.request_id`，可據此撈出該次請求的完整軌跡（舊資料為 NULL）
    pub request_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// `GET /admin/audit_logs` 的篩選參數。
///
/// 放在 structs/ 而非 route 內是為了讓 repository 的 list 與 count 共用同一份
/// 篩選條件（`repositories/audit_logs.rs` 的 `AUDIT_FILTER`），不必把 6 個參數平鋪兩次
/// —— 兩邊 WHERE 一旦漂移，`total` 就會與實際筆數對不上（範本同 `structs/logs.rs`）。
#[derive(Deserialize, Default)]
pub struct AuditLogQuery {
    pub user_email: Option<String>,
    pub method: Option<String>,
    /// 路徑模糊比對
    pub path: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    /// `admin` / `member`（不給 = 兩者都列）。member 的稽核是 2026-08-09 才開始記的，
    /// 在那之前的列一律是 admin。
    pub actor_type: Option<String>,
}
