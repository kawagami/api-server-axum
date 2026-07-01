//! 統一發票對獎 — 純邏輯（零 IO，可單測）+ 中獎號碼來源抓取。
//!
//! 對獎引擎不在意號碼怎麼來：`PeriodNumbers` 由開放資料抓取或 admin 手動輸入組成，
//! 一律經 `match_prize` 比對。比對只看發票號碼後 8 碼（字軌前 2 英文不參與）。

use crate::errors::AppError;
use chrono::{Datelike, NaiveDate};
use regex::Regex;
use reqwest::Client;

/// 財政部統一發票中獎號碼 RSS
const FEED_URL: &str = "https://invoice.etax.nat.gov.tw/invoice.xml";

/// 某一期的中獎號碼
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PeriodNumbers {
    pub special: Option<String>, // 特別獎（8 碼）
    pub grand: Option<String>,   // 特獎（8 碼）
    pub first: Vec<String>,      // 頭獎（8 碼，通常 3 組）
    pub additional: Vec<String>, // 增開六獎（3 碼，0~N 組）
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrizeTier {
    Special,
    Grand,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    AdditionalSixth,
}

impl PrizeTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Special => "special",
            Self::Grand => "grand",
            Self::First => "first",
            Self::Second => "second",
            Self::Third => "third",
            Self::Fourth => "fourth",
            Self::Fifth => "fifth",
            Self::Sixth => "sixth",
            Self::AdditionalSixth => "additional_sixth",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        Some(match s {
            "special" => Self::Special,
            "grand" => Self::Grand,
            "first" => Self::First,
            "second" => Self::Second,
            "third" => Self::Third,
            "fourth" => Self::Fourth,
            "fifth" => Self::Fifth,
            "sixth" => Self::Sixth,
            "additional_sixth" => Self::AdditionalSixth,
            _ => return None,
        })
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Special => "特別獎",
            Self::Grand => "特獎",
            Self::First => "頭獎",
            Self::Second => "二獎",
            Self::Third => "三獎",
            Self::Fourth => "四獎",
            Self::Fifth => "五獎",
            Self::Sixth => "六獎",
            Self::AdditionalSixth => "增開六獎",
        }
    }

    /// 獎金（新臺幣）
    pub fn amount(&self) -> i64 {
        match self {
            Self::Special => 10_000_000,
            Self::Grand => 2_000_000,
            Self::First => 200_000,
            Self::Second => 40_000,
            Self::Third => 10_000,
            Self::Fourth => 4_000,
            Self::Fifth => 1_000,
            Self::Sixth | Self::AdditionalSixth => 200,
        }
    }

    /// 名次（越大越高），用於取最高獎
    fn rank(&self) -> u8 {
        match self {
            Self::Special => 9,
            Self::Grand => 8,
            Self::First => 7,
            Self::Second => 6,
            Self::Third => 5,
            Self::Fourth => 4,
            Self::Fifth => 3,
            Self::Sixth => 2,
            Self::AdditionalSixth => 1,
        }
    }
}

/// 比對單張發票，回傳命中的**最高**獎別（None = 未中）
pub fn match_prize(invoice_number: &str, n: &PeriodNumbers) -> Option<PrizeTier> {
    if invoice_number.len() < 8 {
        return None;
    }
    let d = &invoice_number[invoice_number.len() - 8..]; // 後 8 碼
    if !d.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    if n.special.as_deref() == Some(d) {
        return Some(PrizeTier::Special);
    }
    if n.grand.as_deref() == Some(d) {
        return Some(PrizeTier::Grand);
    }

    let mut best: Option<PrizeTier> = None;
    for f in &n.first {
        if f.len() != 8 {
            continue;
        }
        if d == f.as_str() {
            return Some(PrizeTier::First); // 全中即頭獎，已是 first 系列最高
        }
        // 由長到短取最長相符尾數
        for (len, tier) in [
            (7, PrizeTier::Second),
            (6, PrizeTier::Third),
            (5, PrizeTier::Fourth),
            (4, PrizeTier::Fifth),
            (3, PrizeTier::Sixth),
        ] {
            if d[8 - len..] == f[8 - len..] {
                best = keep_higher(best, tier);
                break;
            }
        }
    }
    for a in &n.additional {
        if !a.is_empty() && a.len() <= 8 && d.ends_with(a.as_str()) {
            best = keep_higher(best, PrizeTier::AdditionalSixth);
        }
    }
    best
}

fn keep_higher(cur: Option<PrizeTier>, candidate: PrizeTier) -> Option<PrizeTier> {
    match cur {
        Some(x) if x.rank() >= candidate.rank() => Some(x),
        _ => Some(candidate),
    }
}

/// 由開立日推算對獎期別 key 'YYYYMM'（期末偶數月）
pub fn period_of_date(date: NaiveDate) -> String {
    let m = date.month();
    let ending = if m % 2 == 1 { m + 1 } else { m };
    format!("{:04}{:02}", date.year(), ending)
}

/// 解析財政部 RSS，回傳各期 (period, PeriodNumbers)。純函式，可測。
pub fn parse_feed(xml: &str) -> Vec<(String, PeriodNumbers)> {
    let item_re = Regex::new(r"(?s)<item>(.*?)</item>").unwrap();
    let title_re = Regex::new(r"(?s)<title>(.*?)</title>").unwrap();
    let period_re = Regex::new(r"(\d+)\s*年\s*0*(\d+)\s*[~～-]\s*0*(\d+)\s*月").unwrap();
    let special_re = Regex::new(r"特別獎\D*?(\d{8})").unwrap();
    let grand_re = Regex::new(r"特獎\D*?(\d{8})").unwrap();
    let first_block_re = Regex::new(r"頭獎[^0-9]*([0-9、,\s]+)").unwrap();
    let add_block_re = Regex::new(r"增開六獎[^0-9]*([0-9、,\s]+)").unwrap();
    let d8 = Regex::new(r"\d{8}").unwrap();
    let d3 = Regex::new(r"\d{3}").unwrap();

    let mut out = Vec::new();
    for item in item_re.captures_iter(xml) {
        let body = &item[1];

        let Some(title) = title_re.captures(body) else { continue };
        let Some(pcap) = period_re.captures(title[1].trim()) else { continue };
        let minguo: i32 = pcap[1].parse().unwrap_or(0);
        let ending: u32 = pcap[3].parse().unwrap_or(0);
        if minguo == 0 || ending == 0 {
            continue;
        }
        let period = format!("{:04}{:02}", minguo + 1911, ending);

        let mut nums = PeriodNumbers::default();
        nums.special = special_re
            .captures(body)
            .map(|c| c[1].to_string());
        nums.grand = grand_re.captures(body).map(|c| c[1].to_string());
        if let Some(c) = first_block_re.captures(body) {
            nums.first = d8.find_iter(&c[1]).map(|m| m.as_str().to_string()).collect();
        }
        if let Some(c) = add_block_re.captures(body) {
            nums.additional = d3.find_iter(&c[1]).map(|m| m.as_str().to_string()).collect();
        }
        out.push((period, nums));
    }
    out
}

/// 抓取中獎號碼（IO；薄包裝，解析交給純函式 `parse_feed`）
pub async fn fetch_winning_numbers(client: &Client) -> Result<Vec<(String, PeriodNumbers)>, AppError> {
    let text = client.get(FEED_URL).send().await?.text().await?;
    Ok(parse_feed(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nums() -> PeriodNumbers {
        PeriodNumbers {
            special: Some("47406327".to_string()),
            grand: Some("05579058".to_string()),
            first: vec![
                "49912232".to_string(),
                "73145004".to_string(),
                "99174704".to_string(),
            ],
            additional: vec!["123".to_string()],
        }
    }

    #[test]
    fn special_prize() {
        assert_eq!(match_prize("AB47406327", &nums()), Some(PrizeTier::Special));
    }

    #[test]
    fn grand_prize() {
        assert_eq!(match_prize("XY05579058", &nums()), Some(PrizeTier::Grand));
    }

    #[test]
    fn first_prize_full_match() {
        assert_eq!(match_prize("ZZ73145004", &nums()), Some(PrizeTier::First));
    }

    #[test]
    fn second_prize_last7() {
        // 末 7 碼同 49912232 → 9912232
        assert_eq!(match_prize("AB19912232", &nums()), Some(PrizeTier::Second));
    }

    #[test]
    fn sixth_prize_last3() {
        // 末 3 碼同頭獎 ...232
        assert_eq!(match_prize("AB55555232", &nums()), Some(PrizeTier::Sixth));
    }

    #[test]
    fn additional_sixth_last3() {
        // 末 3 碼 123，不中任何頭獎尾數
        assert_eq!(
            match_prize("AB78900123", &nums()),
            Some(PrizeTier::AdditionalSixth)
        );
    }

    #[test]
    fn no_prize() {
        assert_eq!(match_prize("AB88888888", &nums()), None);
    }

    #[test]
    fn letters_do_not_affect_match() {
        // 字軌不同但號碼相同，仍中特別獎
        assert_eq!(match_prize("QQ47406327", &nums()), Some(PrizeTier::Special));
    }

    #[test]
    fn highest_tier_wins_when_multiple() {
        // 完全等於頭獎 → First，而非僅尾數的低獎
        assert_eq!(match_prize("AB49912232", &nums()), Some(PrizeTier::First));
    }

    #[test]
    fn period_calc() {
        assert_eq!(
            period_of_date(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
            "202602"
        );
        assert_eq!(
            period_of_date(NaiveDate::from_ymd_opt(2026, 6, 22).unwrap()),
            "202606"
        );
        assert_eq!(
            period_of_date(NaiveDate::from_ymd_opt(2026, 11, 1).unwrap()),
            "202612"
        );
    }

    #[test]
    fn parse_feed_extracts_period_and_numbers() {
        let xml = r#"
        <rss version="2.0"><channel>
          <item>
            <title>114年 05~06月</title>
            <description>&lt;p&gt;特別獎：47406327&lt;/p&gt;&lt;p&gt;特獎：05579058&lt;/p&gt;&lt;p&gt;頭獎：49912232、73145004、99174704&lt;/p&gt;&lt;p&gt;增開六獎：728、990&lt;/p&gt;</description>
          </item>
        </channel></rss>
        "#;
        let parsed = parse_feed(xml);
        assert_eq!(parsed.len(), 1);
        let (period, n) = &parsed[0];
        assert_eq!(period, "202506");
        assert_eq!(n.special.as_deref(), Some("47406327"));
        assert_eq!(n.grand.as_deref(), Some("05579058"));
        assert_eq!(n.first, vec!["49912232", "73145004", "99174704"]);
        assert_eq!(n.additional, vec!["728", "990"]);
    }
}
