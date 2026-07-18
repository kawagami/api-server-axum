use std::collections::HashSet;

/// instance 級可開關功能 — `enabled_features` 設定的唯一 key 權威。
/// 新增功能：加 variant + `as_str` / `from_key` match arm + `ALL`，
/// route 掛 `with_feature`、job 補 `AppJob::feature()`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    Blog,
    Tools,
    Roster,
    Games,
    Stocks,
    Portfolio,
    Ledger,
    Invoices,
    Lotto,
    Vocab,
    Torrents,
    GovTenders,
}

impl Feature {
    pub const ALL: &'static [Feature] = &[
        Feature::Blog,
        Feature::Tools,
        Feature::Roster,
        Feature::Games,
        Feature::Stocks,
        Feature::Portfolio,
        Feature::Ledger,
        Feature::Invoices,
        Feature::Lotto,
        Feature::Vocab,
        Feature::Torrents,
        Feature::GovTenders,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Feature::Blog => "blog",
            Feature::Tools => "tools",
            Feature::Roster => "roster",
            Feature::Games => "games",
            Feature::Stocks => "stocks",
            Feature::Portfolio => "portfolio",
            Feature::Ledger => "ledger",
            Feature::Invoices => "invoices",
            Feature::Lotto => "lotto",
            Feature::Vocab => "vocab",
            Feature::Torrents => "torrents",
            Feature::GovTenders => "gov_tenders",
        }
    }

    pub fn from_key(key: &str) -> Option<Feature> {
        Feature::ALL.iter().copied().find(|f| f.as_str() == key)
    }

    /// 解析 `enabled_features` 設定值成白名單。
    /// `None` = 全開（值為 `all`）；壞值 fail-open 回 `None` 並記 error
    /// （PATCH 端已嚴格驗證，此處只防手改 DB 弄壞站）。
    pub fn parse_setting(value: &str) -> Option<HashSet<Feature>> {
        if value == "all" {
            return None;
        }
        match serde_json::from_str::<Vec<String>>(value) {
            Ok(keys) => Some(keys.iter().filter_map(|k| Feature::from_key(k)).collect()),
            Err(e) => {
                tracing::error!("enabled_features 設定值無法解析，視為全開: {e}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_keys_roundtrip() {
        for f in Feature::ALL {
            assert_eq!(Feature::from_key(f.as_str()), Some(*f));
        }
        assert_eq!(Feature::from_key("not_a_feature"), None);
    }

    #[test]
    fn parse_setting_all_and_list() {
        assert_eq!(Feature::parse_setting("all"), None);
        let set = Feature::parse_setting(r#"["blog","tools"]"#).unwrap();
        assert_eq!(set.len(), 2);
        assert!(set.contains(&Feature::Blog));
        assert!(set.contains(&Feature::Tools));
        assert!(!set.contains(&Feature::Games));
    }

    #[test]
    fn parse_setting_bad_value_fails_open() {
        assert_eq!(Feature::parse_setting("not json"), None);
        // 未知 key 靜默忽略（值已過 PATCH 驗證，此處寬鬆）
        let set = Feature::parse_setting(r#"["blog","unknown"]"#).unwrap();
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn parse_setting_empty_list_disables_everything() {
        let set = Feature::parse_setting("[]").unwrap();
        assert!(set.is_empty());
    }
}
