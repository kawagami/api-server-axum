/// 修剪選填字串，空白/空字串視為 None。
///
/// 「使用者送 `""` 或 `"   "`」與「沒送這個欄位」在業務語意上是同一件事（匿名留言、
/// 選填備註），但 serde 收到的是 `Some("")`，直接落 DB 會讓「空字串」與 NULL 兩種
/// 「沒填」並存 —— 查詢與顯示端就得各自處理兩種。統一在入口收成 None。
///
/// 收斂前 `services/{messages,blog_comments}.rs` 各有一份逐字相同的私有版本。
pub fn normalize_optional(s: Option<String>) -> Option<String> {
    s.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::normalize_optional;

    #[test]
    fn trims_and_keeps_non_empty() {
        assert_eq!(normalize_optional(Some("  kawa  ".into())), Some("kawa".into()));
    }

    /// 空字串與純空白都要收成 None —— 否則 DB 裡「沒填」會有兩種表示法
    #[test]
    fn blank_becomes_none() {
        assert_eq!(normalize_optional(Some("".into())), None);
        assert_eq!(normalize_optional(Some("   \t ".into())), None);
    }

    #[test]
    fn none_stays_none() {
        assert_eq!(normalize_optional(None), None);
    }
}
