use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

/// 台北時區偏移（UTC+8，無 DST）。
///
/// 「今天」在這個專案裡一律以台北日為準：資料來源（TWSE / 台彩 / 財政部）都照台北營業日
/// 發佈，統計的日界也是台北 00:00。
///
/// **為什麼不用 `chrono::Local`**：它的結果取決於行程的 `TZ` 環境變數。生產 image 目前
/// 有設（`Dockerfile` 的 `ENV TZ=Asia/Taipei`），但那條規則發生在部署層、不在版控的
/// Rust 程式碼裡 —— 拿掉或換基底 image 少了 tzdata，`Local` 就悄悄退回 UTC，
/// 台北 00:00–08:00 那八小時的「今天」全部算成昨天，而且沒有任何編譯期或啟動期徵兆。
/// 明寫偏移量不依賴環境。（歷史：導入這幾支時 image 確實沒設 `TZ`，`Local` 當時等於 UTC，
/// `services/portfolio.rs` ×3、`services/stocks.rs`、`jobs/fetch_buyback_periods.rs` 都中過。）
///
/// **偏移量只在這裡出現一次**：不要再寫第二個 `FixedOffset::east_opt(8 * 3600)`，也不要在
/// 任何地方用 `Local::now()` 當「今天」。
pub fn taipei_offset() -> FixedOffset {
    FixedOffset::east_opt(8 * 3600).expect("UTC+8 是合法偏移")
}

/// 台北時間的「現在」
pub fn taipei_now() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&taipei_offset())
}

/// 台北時間的「今天」
pub fn taipei_today() -> NaiveDate {
    taipei_now().date_naive()
}

/// 解析民國日期字串（如 "114/06/10"）為西元 NaiveDate
pub fn parse_roc_date(s: &str) -> Option<NaiveDate> {
    let parts: Vec<&str> = s.trim().split('/').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i32 = parts[0].trim().parse().ok()?;
    let month: u32 = parts[1].trim().parse().ok()?;
    let day: u32 = parts[2].trim().parse().ok()?;
    NaiveDate::from_ymd_opt(year + 1911, month, day)
}

/// 解析無分隔民國日期（如 "1150625" = 115/06/25）為西元 NaiveDate。
/// 末 4 碼為 MMDD，其餘為民國年。
pub fn parse_roc_compact_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.len() < 5 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (year_str, md) = s.split_at(s.len() - 4);
    let year: i32 = year_str.parse().ok()?;
    let month: u32 = md[..2].parse().ok()?;
    let day: u32 = md[2..].parse().ok()?;
    NaiveDate::from_ymd_opt(year + 1911, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 守住「台北 = UTC+8」這件事本身：偏移寫錯會讓所有日界（訪客統計、排行榜週界、
    /// 對獎的當日判斷）整批偏移，而那種錯不會有任何編譯或執行期徵兆。
    #[test]
    fn taipei_is_utc_plus_eight() {
        assert_eq!(taipei_offset().local_minus_utc(), 8 * 3600);
    }

    /// 台北日必為 UTC 日或其後一日 —— 不可能落在 UTC 日之前。
    #[test]
    fn taipei_today_is_never_behind_utc() {
        let utc_today = Utc::now().date_naive();
        let diff = (taipei_today() - utc_today).num_days();
        assert!((0..=1).contains(&diff), "台北日與 UTC 日相差 {diff} 天");
    }

    #[test]
    fn parses_compact_roc_date() {
        assert_eq!(
            parse_roc_compact_date("1150625"),
            NaiveDate::from_ymd_opt(2026, 6, 25)
        );
    }

    #[test]
    fn rejects_malformed_compact_roc_date() {
        assert_eq!(parse_roc_compact_date("日期"), None);
        assert_eq!(parse_roc_compact_date(""), None);
        assert_eq!(parse_roc_compact_date("1151325"), None); // 月份 13 非法
    }
}
