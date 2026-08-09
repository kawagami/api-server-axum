use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `logs` 表的一列，同時是 `GET /logs` 與 `GET /logs/request/{id}` 的回應型別。
#[derive(Serialize, sqlx::FromRow)]
pub struct Log {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub target: String,
    pub file: Option<String>,
    pub line: Option<i32>,
    /// 對應 `x-request-id` / 錯誤 body 的 `request_id`，用來把一個請求的 log 串起來
    pub request_id: Option<String>,
    /// event 與 span 的其餘 field（`self` = 錯誤細節、`method`、`path`…），見 `logging.rs`
    pub fields: Option<Value>,
    pub created_at: DateTime<Utc>,
}

/// `GET /logs` 的篩選參數。
///
/// 放在 structs/ 而非 route 內是為了讓 repository 的 list 與 count 能共用同一份
/// 篩選條件（`repositories/logs.rs` 的 `LOG_FILTER`），不必把 6 個參數平鋪兩次。
#[derive(Deserialize)]
pub struct LogQuery {
    /// 可帶多個，逗號分隔（`level=WARN,ERROR`）；單值仍相容既有前端呼叫
    pub level: Option<String>,
    /// message 或 fields 的模糊比對（錯誤細節在 fields 裡，見 `logging.rs`）
    pub q: Option<String>,
    /// tracing target 模糊比對（例如 `api_server_axum::errors` / `sqlx`）
    pub target: Option<String>,
    pub request_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl LogQuery {
    /// `level=WARN,ERROR` → `["WARN","ERROR"]`；全空白 / 空字串當作沒帶。
    /// 一律轉大寫 —— DB 存的是 `meta.level()` 的大寫字面值，容忍呼叫端寫小寫。
    pub fn levels(&self) -> Option<Vec<String>> {
        let levels: Vec<String> = self
            .level
            .as_deref()?
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(str::to_uppercase)
            .collect();

        (!levels.is_empty()).then_some(levels)
    }
}

#[cfg(test)]
mod tests {
    use super::LogQuery;

    fn with_level(level: Option<&str>) -> LogQuery {
        LogQuery {
            level: level.map(str::to_owned),
            q: None,
            target: None,
            request_id: None,
            from: None,
            to: None,
        }
    }

    #[test]
    fn levels_handles_single_multi_and_blank() {
        assert_eq!(with_level(None).levels(), None);
        assert_eq!(with_level(Some("")).levels(), None);
        assert_eq!(with_level(Some(" , ")).levels(), None);
        assert_eq!(
            with_level(Some("ERROR")).levels(),
            Some(vec!["ERROR".to_string()])
        );
        assert_eq!(
            with_level(Some("warn, error")).levels(),
            Some(vec!["WARN".to_string(), "ERROR".to_string()])
        );
    }
}
