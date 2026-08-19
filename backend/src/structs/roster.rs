use serde::{Deserialize, Serialize};

/// 排班天數上限。前端 UI 本來就只開放 1–31（一個月），後端跟著收 ——
/// `days` 是 u32，不擋的話一發 `{"days":50000000}`（body 約 50 bytes）就會配置
/// 數 GB 的 Vec<String>，在 1 核 1G 的機器上直接觸發 OOM killer。端點無認證。
pub const MAX_DAYS: u32 = 31;
/// 人數上限。names 與 days 相乘放大，兩邊都要有界。
pub const MAX_NAMES: usize = 100;
/// 單一姓名長度上限。
pub const MAX_NAME_LEN: usize = 50;
/// 連續上班天數上限的預設值。沒有這條約束時，「照累計工時挑人」會自然排出
/// 連上十幾天再連休數天的班表 —— 總工時公平，實務上不能用。
pub const DEFAULT_MAX_CONSECUTIVE: u32 = 5;

/// 班別字串是 API 契約（前端 `shift-badge.tsx` 以此為 key 查樣式與 i18n），
/// **不可改成英文代碼**，要 i18n 的是前端渲染出來的文字。
pub const SHIFT_MORNING: &str = "早班";
pub const SHIFT_NIGHT: &str = "晚班";
pub const SHIFT_OFF: &str = "休";

/// 排班規則。**用 enum 而不是 String**：舊版收 `String` 再 `match` 字串，未知值
/// （前端改錯 key、手打 typo）會靜默 fallback 成 fairness，使用者拿到一份不是自己
/// 要求的班表卻沒有任何徵兆。改成 enum 之後 serde 解不出來就是 422。
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RosterRule {
    Fairness,
    MorningHeavy,
    NightHeavy,
}

#[derive(Deserialize)]
pub struct RosterRequest {
    pub names: Vec<String>,
    pub days: u32,
    pub rule: RosterRule,
    /// 每日早班需要幾人。與 `night_slots` 是**兩者皆給或皆不給**：不給就由 `rule`
    /// 的權重推算（見 `services::roster::resolve_plan`）。這兩個值是本次改版的重點 ——
    /// 舊版每日人力是「每人各跑 pattern」的副作用，會排出某天某班 0 人的班表。
    #[serde(default)]
    pub morning_slots: Option<u32>,
    /// 每日晚班需要幾人。
    #[serde(default)]
    pub night_slots: Option<u32>,
    /// 連續上班天數上限，省略用 `DEFAULT_MAX_CONSECUTIVE`。
    #[serde(default)]
    pub max_consecutive: Option<u32>,
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
            return Err(format!(
                "姓名長度上限 {MAX_NAME_LEN} 字（「{}…」過長）",
                bad.chars().take(10).collect::<String>()
            ));
        }
        if self.days == 0 || self.days > MAX_DAYS {
            return Err(format!("days 需在 1..={MAX_DAYS}"));
        }
        // 只給一邊會讓另一邊「自動推算」的語意變得無從定義（推算是照 rule 分三份，
        // 不是「補滿剩下的人」），所以直接擋掉而不是猜。
        match (self.morning_slots, self.night_slots) {
            (Some(m), Some(n)) => {
                let people = self.names.len() as u32;
                if m + n == 0 {
                    return Err("morning_slots 與 night_slots 不能同時為 0".to_string());
                }
                if m + n > people {
                    return Err(format!(
                        "morning_slots + night_slots 不能超過人數（{people}）"
                    ));
                }
            }
            (None, None) => {}
            _ => {
                return Err("morning_slots 與 night_slots 必須同時提供或同時省略".to_string());
            }
        }
        if let Some(mc) = self.max_consecutive {
            if mc == 0 || mc > MAX_DAYS {
                return Err(format!("max_consecutive 需在 1..={MAX_DAYS}"));
            }
        }
        Ok(())
    }
}

/// 排班時實際採用的每日人力配置。**回給前端**：使用者沒指定 slots 時這是推算值，
/// 不回傳的話畫面上就沒有任何地方看得出「今天早班幾人」，人力洞又會變成看不見的東西。
#[derive(Serialize, Clone, Copy, Debug)]
pub struct RosterPlan {
    pub morning_slots: u32,
    pub night_slots: u32,
    pub rest_slots: u32,
    pub max_consecutive: u32,
}

/// 排班結果的機器可讀警告碼。
///
/// **刻意不回中文訊息**：前端規則是「不要把後端錯誤訊息印給使用者」（後端訊息全是
/// 寫死繁中，en / zh-CN 使用者會看到中文），所以這裡只給碼，文案由前端 i18n。
#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RosterWarning {
    /// 每日休假人數為 0：人力剛好等於班表需求，沒有人排得到休。
    Understaffed,
    /// 有班別每日 0 人（人太少，兩班無法同時開）。
    ShiftUncovered,
    /// 有人被迫「晚班接早班」（可上班的人不夠，只能違反）。
    NightToMorning,
    /// 有人被迫超過連續上班上限。
    MaxConsecutiveExceeded,
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
    pub plan: RosterPlan,
    pub warnings: Vec<RosterWarning>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(names: Vec<&str>, days: u32) -> RosterRequest {
        RosterRequest {
            names: names.into_iter().map(str::to_string).collect(),
            days,
            rule: RosterRule::Fairness,
            morning_slots: None,
            night_slots: None,
            max_consecutive: None,
        }
    }

    #[test]
    fn accepts_normal_input() {
        assert!(req(vec!["a", "b"], 31).validate().is_ok());
        assert!(req(vec!["a"], 1).validate().is_ok());
    }

    /// 這條是當初修復的重點：未認證的單一請求曾可用巨大 days 觸發整機 OOM
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

    #[test]
    fn slots_must_come_in_pair() {
        let mut r = req(vec!["a", "b", "c"], 7);
        r.morning_slots = Some(1);
        assert!(r.validate().is_err());
        r.night_slots = Some(1);
        assert!(r.validate().is_ok());
    }

    /// slots 總和超過人數就排不出來，先擋在 handler 之外
    #[test]
    fn rejects_slots_over_headcount() {
        let mut r = req(vec!["a", "b"], 7);
        r.morning_slots = Some(2);
        r.night_slots = Some(1);
        assert!(r.validate().is_err());
        r.morning_slots = Some(0);
        r.night_slots = Some(0);
        assert!(r.validate().is_err());
    }

    #[test]
    fn rejects_bad_max_consecutive() {
        let mut r = req(vec!["a", "b"], 7);
        r.max_consecutive = Some(0);
        assert!(r.validate().is_err());
        r.max_consecutive = Some(MAX_DAYS + 1);
        assert!(r.validate().is_err());
        r.max_consecutive = Some(5);
        assert!(r.validate().is_ok());
    }

    /// rule 是 enum，未知字串在 serde 那層就是 422，不會靜默 fallback 成 fairness
    #[test]
    fn unknown_rule_fails_to_deserialize() {
        let body = r#"{"names":["a"],"days":7,"rule":"whatever"}"#;
        assert!(serde_json::from_str::<RosterRequest>(body).is_err());
        let ok = r#"{"names":["a"],"days":7,"rule":"night_heavy"}"#;
        assert!(serde_json::from_str::<RosterRequest>(ok).is_ok());
    }
}
