use serde::{Deserialize, Serialize};

/// 排班天數上限。前端 UI 本來就只開放 1–31（一個月），後端跟著收 ——
/// `days` 是 u32，不擋的話一發 `{"days":50000000}`（body 約 50 bytes）就會配置
/// 數 GB 的 Vec<String>，在 1 核 1G 的機器上直接觸發 OOM killer。端點無認證。
pub const MAX_DAYS: u32 = 31;
/// 人數上限。names 與 days 相乘放大，兩邊都要有界。
pub const MAX_NAMES: usize = 100;
/// 單一姓名長度上限。
pub const MAX_NAME_LEN: usize = 50;

#[derive(Deserialize)]
pub struct RosterRequest {
    pub names: Vec<String>,
    pub days: u32,
    pub rule: String,
}

impl RosterRequest {
    /// 純函式驗證，可測。回 Err 時 handler 直接回 422，絕不進配置迴圈。
    pub fn validate(&self) -> Result<(), String> {
        if self.names.is_empty() {
            return Err("names 至少需要一位人員".to_string());
        }
        if self.names.len() > MAX_NAMES {
            return Err(format!("names 上限 {MAX_NAMES} 位"));
        }
        if let Some(bad) = self.names.iter().find(|n| n.chars().count() > MAX_NAME_LEN) {
            return Err(format!("姓名長度上限 {MAX_NAME_LEN} 字（「{}…」過長）", bad.chars().take(10).collect::<String>()));
        }
        if self.days == 0 || self.days > MAX_DAYS {
            return Err(format!("days 需在 1..={MAX_DAYS}"));
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct StaffShift {
    pub id: usize,
    pub name: String,
    pub shifts: Vec<String>,
}

#[derive(Serialize)]
pub struct RosterResponse {
    pub status: String,
    pub data: Vec<StaffShift>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(names: Vec<&str>, days: u32) -> RosterRequest {
        RosterRequest {
            names: names.into_iter().map(str::to_string).collect(),
            days,
            rule: "fairness".to_string(),
        }
    }

    #[test]
    fn accepts_normal_input() {
        assert!(req(vec!["a", "b"], 31).validate().is_ok());
        assert!(req(vec!["a"], 1).validate().is_ok());
    }

    /// 這條是本次修復的重點：未認證的單一請求曾可用巨大 days 觸發整機 OOM
    #[test]
    fn rejects_huge_days() {
        assert!(req(vec!["a"], u32::MAX).validate().is_err());
        assert!(req(vec!["a"], 50_000_000).validate().is_err());
        assert!(req(vec!["a"], MAX_DAYS + 1).validate().is_err());
    }

    #[test]
    fn rejects_zero_days() {
        assert!(req(vec!["a"], 0).validate().is_err());
    }

    #[test]
    fn rejects_empty_or_oversized_names() {
        assert!(req(vec![], 7).validate().is_err());
        let many: Vec<&str> = vec!["a"; MAX_NAMES + 1];
        assert!(req(many, 7).validate().is_err());
    }

    #[test]
    fn rejects_overlong_name() {
        let long = "字".repeat(MAX_NAME_LEN + 1);
        assert!(req(vec![long.as_str()], 7).validate().is_err());
    }

    /// 長度用字元數而非 bytes：中文姓名不該因為 UTF-8 佔 3 bytes 就被誤擋
    #[test]
    fn name_length_counts_chars_not_bytes() {
        let cjk = "字".repeat(MAX_NAME_LEN);
        assert!(req(vec![cjk.as_str()], 7).validate().is_ok());
    }
}
