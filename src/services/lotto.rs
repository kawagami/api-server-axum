//! 大樂透 / 威力彩 對獎純引擎（零 IO，可單測）+ 台彩開獎號碼抓取。
//!
//! 對獎只看號碼集合：玩家選號 vs 當期開出號碼。`match_draw` 依 game 分派，
//! 回傳命中的**最高**獎別（None = 未中）。號碼來源（台彩 API）由 `fetch_draws`
//! 取得，解析交給純函式 `parse_draws`。

use crate::{errors::AppError, structs::lotto::Draw};
use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;

pub const LOTTO649: &str = "lotto649";
pub const SUPER638: &str = "super_lotto638";

const API_BASE: &str = "https://api.taiwanlottery.com/TLCAPIWeB/Lottery";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrizeTier {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Ninth,
    General,
}

impl PrizeTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Second => "second",
            Self::Third => "third",
            Self::Fourth => "fourth",
            Self::Fifth => "fifth",
            Self::Sixth => "sixth",
            Self::Seventh => "seventh",
            Self::Eighth => "eighth",
            Self::Ninth => "ninth",
            Self::General => "general",
        }
    }

    pub fn from_db(s: &str) -> Option<Self> {
        Some(match s {
            "first" => Self::First,
            "second" => Self::Second,
            "third" => Self::Third,
            "fourth" => Self::Fourth,
            "fifth" => Self::Fifth,
            "sixth" => Self::Sixth,
            "seventh" => Self::Seventh,
            "eighth" => Self::Eighth,
            "ninth" => Self::Ninth,
            "general" => Self::General,
            _ => return None,
        })
    }

    /// 中文獎別（兩彩種同 key 名稱一致）
    pub fn label(&self) -> &'static str {
        match self {
            Self::First => "頭獎",
            Self::Second => "貳獎",
            Self::Third => "參獎",
            Self::Fourth => "肆獎",
            Self::Fifth => "伍獎",
            Self::Sixth => "陸獎",
            Self::Seventh => "柒獎",
            Self::Eighth => "捌獎",
            Self::Ninth => "玖獎",
            Self::General => "普獎",
        }
    }
}

/// 依 game 分派對獎
pub fn match_draw(
    game: &str,
    picks: &[i16],
    second: Option<i16>,
    main: &[i16],
    special: i16,
) -> Option<PrizeTier> {
    match game {
        LOTTO649 => match_lotto649(picks, main, special),
        SUPER638 => match_super638(picks, second, main, special),
        _ => None,
    }
}

/// 大樂透：玩家 6 號 vs 一般號 6 + 特別號。
/// `m` = 一般號命中數、`s` = 玩家某選號 = 特別號（特別號取自剩餘池，不與一般號重疊）。
pub fn match_lotto649(picks: &[i16], main: &[i16], special: i16) -> Option<PrizeTier> {
    let m = picks.iter().filter(|p| main.contains(p)).count();
    let s = picks.contains(&special);
    Some(match (m, s) {
        (6, _) => PrizeTier::First,
        (5, true) => PrizeTier::Second,
        (5, false) => PrizeTier::Third,
        (4, true) => PrizeTier::Fourth,
        (4, false) => PrizeTier::Fifth,
        (3, true) => PrizeTier::Sixth,
        (3, false) => PrizeTier::General,
        (2, true) => PrizeTier::Seventh,
        _ => return None,
    })
}

/// 威力彩：玩家第一區 6 + 第二區 1 vs 第一區 6 + 第二區 1。
/// `f` = 第一區命中數、`b` = 中第二區。
pub fn match_super638(
    picks: &[i16],
    second: Option<i16>,
    main: &[i16],
    draw_second: i16,
) -> Option<PrizeTier> {
    let f = picks.iter().filter(|p| main.contains(p)).count();
    let b = second == Some(draw_second);
    Some(match (f, b) {
        (6, true) => PrizeTier::First,
        (6, false) => PrizeTier::Second,
        (5, true) => PrizeTier::Third,
        (5, false) => PrizeTier::Fourth,
        (4, true) => PrizeTier::Fifth,
        (4, false) => PrizeTier::Sixth,
        (3, true) => PrizeTier::Seventh,
        (2, true) => PrizeTier::Eighth,
        (3, false) => PrizeTier::Ninth,
        _ => return None,
    })
}

// ── 台彩開獎號碼抓取 ──────────────────────────────────────────

#[derive(Deserialize)]
struct ApiResp {
    content: Option<ApiContent>,
}

#[derive(Deserialize)]
struct ApiContent {
    #[serde(rename = "lotto649Res")]
    lotto649: Option<Vec<ApiDraw>>,
    #[serde(rename = "superLotto638Res")]
    super638: Option<Vec<ApiDraw>>,
}

#[derive(Deserialize)]
struct ApiDraw {
    period: i64,
    #[serde(rename = "lotteryDate")]
    lottery_date: String, // "2026-05-29T00:00:00"
    #[serde(rename = "drawNumberSize")]
    draw_number_size: Vec<i16>, // [0:6] 一般號/第一區（已排序）+ [6] 特別號/第二區
}

/// 解析台彩 API 回應為 Draw 列表（純函式，可測）
pub fn parse_draws(game: &str, body: &str) -> Result<Vec<Draw>, AppError> {
    let resp: ApiResp = serde_json::from_str(body)?;
    let Some(content) = resp.content else {
        return Ok(vec![]);
    };
    let raw = match game {
        LOTTO649 => content.lotto649,
        SUPER638 => content.super638,
        _ => None,
    }
    .unwrap_or_default();

    let mut out = Vec::new();
    for d in raw {
        if d.draw_number_size.len() < 7 {
            continue;
        }
        let Ok(draw_date) = NaiveDate::parse_from_str(&d.lottery_date[..10.min(d.lottery_date.len())], "%Y-%m-%d")
        else {
            continue;
        };
        out.push(Draw {
            game: game.to_string(),
            period: d.period.to_string(),
            draw_date,
            main_nums: d.draw_number_size[..6].to_vec(),
            special: d.draw_number_size[6],
        });
    }
    Ok(out)
}

/// 抓某彩種某月的開獎結果（IO；薄包裝，解析交給純函式 `parse_draws`）
pub async fn fetch_draws(client: &Client, game: &str, month: &str) -> Result<Vec<Draw>, AppError> {
    let path = match game {
        LOTTO649 => "Lotto649Result",
        SUPER638 => "SuperLotto638Result",
        _ => return Ok(vec![]),
    };
    let url = format!("{API_BASE}/{path}?month={month}&pageSize=31");
    let body = client.get(&url).send().await?.text().await?;
    parse_draws(game, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 大樂透：一般號 [1,2,3,4,5,6]，特別號 7
    const MAIN: &[i16] = &[1, 2, 3, 4, 5, 6];
    const SPECIAL: i16 = 7;

    #[test]
    fn lotto649_first() {
        assert_eq!(match_lotto649(&[1, 2, 3, 4, 5, 6], MAIN, SPECIAL), Some(PrizeTier::First));
    }
    #[test]
    fn lotto649_second_5_plus_special() {
        // 中 5 一般號 + 特別號(7)
        assert_eq!(match_lotto649(&[1, 2, 3, 4, 5, 7], MAIN, SPECIAL), Some(PrizeTier::Second));
    }
    #[test]
    fn lotto649_third_5_no_special() {
        assert_eq!(match_lotto649(&[1, 2, 3, 4, 5, 40], MAIN, SPECIAL), Some(PrizeTier::Third));
    }
    #[test]
    fn lotto649_fourth_4_plus_special() {
        assert_eq!(match_lotto649(&[1, 2, 3, 4, 7, 41], MAIN, SPECIAL), Some(PrizeTier::Fourth));
    }
    #[test]
    fn lotto649_fifth_4_no_special() {
        assert_eq!(match_lotto649(&[1, 2, 3, 4, 40, 41], MAIN, SPECIAL), Some(PrizeTier::Fifth));
    }
    #[test]
    fn lotto649_sixth_3_plus_special() {
        assert_eq!(match_lotto649(&[1, 2, 3, 7, 40, 41], MAIN, SPECIAL), Some(PrizeTier::Sixth));
    }
    #[test]
    fn lotto649_general_3_no_special() {
        assert_eq!(match_lotto649(&[1, 2, 3, 40, 41, 42], MAIN, SPECIAL), Some(PrizeTier::General));
    }
    #[test]
    fn lotto649_seventh_2_plus_special() {
        assert_eq!(match_lotto649(&[1, 2, 7, 40, 41, 42], MAIN, SPECIAL), Some(PrizeTier::Seventh));
    }
    #[test]
    fn lotto649_no_prize() {
        // 中 2 但無特別號 → 未中
        assert_eq!(match_lotto649(&[1, 2, 40, 41, 42, 43], MAIN, SPECIAL), None);
        assert_eq!(match_lotto649(&[40, 41, 42, 43, 44, 45], MAIN, SPECIAL), None);
    }

    // 威力彩：第一區 [1..6]，第二區 3
    #[test]
    fn super638_first() {
        assert_eq!(match_super638(&[1, 2, 3, 4, 5, 6], Some(3), MAIN, 3), Some(PrizeTier::First));
    }
    #[test]
    fn super638_second_6_no_b() {
        assert_eq!(match_super638(&[1, 2, 3, 4, 5, 6], Some(8), MAIN, 3), Some(PrizeTier::Second));
    }
    #[test]
    fn super638_third_5_plus_b() {
        assert_eq!(match_super638(&[1, 2, 3, 4, 5, 30], Some(3), MAIN, 3), Some(PrizeTier::Third));
    }
    #[test]
    fn super638_sixth_4_no_b() {
        assert_eq!(match_super638(&[1, 2, 3, 4, 30, 31], Some(8), MAIN, 3), Some(PrizeTier::Sixth));
    }
    #[test]
    fn super638_seventh_3_plus_b() {
        assert_eq!(match_super638(&[1, 2, 3, 30, 31, 32], Some(3), MAIN, 3), Some(PrizeTier::Seventh));
    }
    #[test]
    fn super638_eighth_2_plus_b() {
        assert_eq!(match_super638(&[1, 2, 30, 31, 32, 33], Some(3), MAIN, 3), Some(PrizeTier::Eighth));
    }
    #[test]
    fn super638_ninth_3_no_b() {
        assert_eq!(match_super638(&[1, 2, 3, 30, 31, 32], Some(8), MAIN, 3), Some(PrizeTier::Ninth));
    }
    #[test]
    fn super638_no_prize() {
        // 只中第二區 → 未中
        assert_eq!(match_super638(&[30, 31, 32, 33, 34, 35], Some(3), MAIN, 3), None);
        // 中 1 第一區 + 第二區 → 未中
        assert_eq!(match_super638(&[1, 30, 31, 32, 33, 34], Some(3), MAIN, 3), None);
    }

    #[test]
    fn parse_lotto649_json() {
        let body = r#"{
            "rtCode": 0, "rtMsg": "OK",
            "content": {
                "totalSize": 1,
                "lotto649Res": [
                    {
                        "period": 115000057,
                        "lotteryDate": "2026-05-29T00:00:00",
                        "drawNumberSize": [4, 11, 23, 30, 38, 46, 7],
                        "drawNumberAppear": [38, 4, 46, 11, 30, 23, 7]
                    }
                ]
            }
        }"#;
        let draws = parse_draws(LOTTO649, body).unwrap();
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].period, "115000057");
        assert_eq!(draws[0].draw_date, NaiveDate::from_ymd_opt(2026, 5, 29).unwrap());
        assert_eq!(draws[0].main_nums, vec![4, 11, 23, 30, 38, 46]);
        assert_eq!(draws[0].special, 7);
    }

    #[test]
    fn parse_wrong_game_array_empty() {
        let body = r#"{"content":{"lotto649Res":[{"period":1,"lotteryDate":"2026-05-29T00:00:00","drawNumberSize":[1,2,3,4,5,6,7]}]}}"#;
        // 要 super638 的陣列，但 JSON 只有 lotto649Res → 空
        assert!(parse_draws(SUPER638, body).unwrap().is_empty());
    }
}
