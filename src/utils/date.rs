use chrono::NaiveDate;

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
