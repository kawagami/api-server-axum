//! 批次操作的共用參數與回應形狀（多選刪除等）。
//!
//! ids 刻意放 **query**（`DELETE /admin/xxx?ids=1,2,3`）而不放 body：`middleware/audit.rs`
//! 只記 method / path / query、不讀 body，放 body 的話稽核只剩一列「有人批次刪了東西」，
//! 查不出刪了哪幾筆。代價是 URL 長度，所以有 `MAX_BATCH_IDS` 上限。

use serde::{Deserialize, Deserializer, Serialize};

/// 單次批次最多幾個 id。100 個 i64 逗號串約 1–2KB，nginx / Cloudflare 的 URL 上限都撐得住；
/// 後台清單一頁 50 筆，兩頁全選剛好用滿。
pub const MAX_BATCH_IDS: usize = 100;

/// `?ids=1,2,3`。解析時去重、排序；空的、非正整數、超過上限都在反序列化階段擋掉
/// （經 `extract::Query` 轉成統一形狀的 400）。
#[derive(Deserialize)]
pub struct IdsQuery {
    #[serde(deserialize_with = "deserialize_ids")]
    pub ids: Vec<i64>,
}

fn deserialize_ids<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<i64>, D::Error> {
    let raw = String::deserialize(d)?;
    parse_ids(&raw).map_err(serde::de::Error::custom)
}

fn parse_ids(raw: &str) -> Result<Vec<i64>, String> {
    let mut ids = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| match s.parse::<i64>() {
            Ok(id) if id > 0 => Ok(id),
            _ => Err(format!("ids 含不合法的值：{s}")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err("ids 不可為空".into());
    }
    if ids.len() > MAX_BATCH_IDS {
        return Err(format!("一次最多 {MAX_BATCH_IDS} 筆"));
    }
    Ok(ids)
}

/// 批次刪除的回應：實際刪掉的 id。
///
/// 不存在的 id 直接略過、不回 404（冪等：另一個分頁先刪掉了，結果一樣是「不在了」），
/// 呼叫端以這份清單為準更新畫面。有 body，所以回 200 而不是單筆刪除的 204。
#[derive(Serialize)]
pub struct BatchDeleted {
    pub deleted: Vec<i64>,
}

#[cfg(test)]
mod tests {
    use super::{parse_ids, MAX_BATCH_IDS};

    #[test]
    fn parses_sorts_and_dedupes() {
        assert_eq!(parse_ids("3,1,2,3").unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn tolerates_spaces_and_trailing_comma() {
        assert_eq!(parse_ids(" 5 , 7,").unwrap(), vec![5, 7]);
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_ids("").is_err());
        assert!(parse_ids(",,").is_err());
    }

    #[test]
    fn rejects_non_positive_and_garbage() {
        assert!(parse_ids("1,0").is_err());
        assert!(parse_ids("1,-2").is_err());
        assert!(parse_ids("1,abc").is_err());
    }

    /// 上限以「去重後」計：重複的 id 不該讓合法請求被擋
    #[test]
    fn limit_counts_unique_ids() {
        let at_limit: Vec<String> = (1..=MAX_BATCH_IDS as i64).map(|i| i.to_string()).collect();
        assert!(parse_ids(&at_limit.join(",")).is_ok());
        let dup = format!("{},1", at_limit.join(","));
        assert!(parse_ids(&dup).is_ok());
        let over = format!("{},{}", at_limit.join(","), MAX_BATCH_IDS + 1);
        assert!(parse_ids(&over).is_err());
    }
}
