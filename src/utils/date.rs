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
