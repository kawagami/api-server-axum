use crate::structs::vocab::{LeaderboardPeriod, QuestionKind};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Utc};
use rand::Rng;

// ---------- 等級曲線 / 經驗值公式(純函式) ----------

/// 升到 level n 所需的累積 exp:100 × (n-1)^1.5,level 1 = 0
pub fn exp_for_level(level: i32) -> i64 {
    if level <= 1 {
        return 0;
    }
    (100.0 * f64::from(level - 1).powf(1.5)).round() as i64
}

pub fn level_for_exp(exp: i64) -> i32 {
    let mut level = 1;
    while exp_for_level(level + 1) <= exp {
        level += 1;
    }
    level
}

/// 單題得分:基礎依難度、combo 連對加成(封頂)、拼字題 ×1.5
/// combo 傳入「本題答對後」的連對數
pub fn answer_exp(difficulty: i16, combo: i32, kind: QuestionKind) -> i64 {
    let base = 10 + i64::from(difficulty - 1) * 5;
    let combo_bonus = i64::from(combo.min(10)) * 2;
    let raw = base + combo_bonus;
    match kind {
        QuestionKind::Choice => raw,
        QuestionKind::Spelling => raw * 3 / 2,
    }
}

/// 生存模式難度曲線:依已答題數決定出題難度區間
fn difficulty_window(answered: i32) -> (i16, i16) {
    match answered {
        0..=9 => (1, 2),
        10..=19 => (1, 3),
        20..=29 => (2, 4),
        _ => (2, 5),
    }
}

/// 難度窗口 clamp 到該語言題庫的實際上下界。
/// 曲線是照英文難度 1–5 全分布調的;日文題庫可能只有 N5+N4(難度 1–2),
/// 不 clamp 的話中後段窗口會整個落在題庫外抽不到字。
pub(super) fn clamped_window(answered: i32, diff_min: i16, diff_max: i16) -> (i16, i16) {
    let (lo, hi) = difficulty_window(answered);
    (lo.clamp(diff_min, diff_max), hi.clamp(diff_min, diff_max))
}

/// 前 5 題全選擇題暖身,之後 30% 出拼字題
pub(super) fn pick_kind(answered: i32) -> QuestionKind {
    if answered < 5 {
        QuestionKind::Choice
    } else if rand::rng().random_bool(0.3) {
        QuestionKind::Spelling
    } else {
        QuestionKind::Choice
    }
}

/// 把例句中的單字挖空(不分大小寫);找不到就不給例句
pub(super) fn mask_sentence(sentence: &str, word: &str) -> Option<String> {
    let lower_sentence = sentence.to_lowercase();
    let lower_word = word.to_lowercase();
    let pos = lower_sentence.find(&lower_word)?;
    let mut masked = String::with_capacity(sentence.len());
    masked.push_str(&sentence[..pos]);
    masked.push_str(&"_".repeat(word.chars().count()));
    masked.push_str(&sentence[pos + lower_word.len()..]);
    Some(masked)
}

/// 限時時長:只接受 3 / 5 / 10 分,其他一律回退 10
pub(super) fn resolve_duration_minutes(m: Option<i64>) -> i64 {
    match m {
        Some(3) => 3,
        Some(5) => 5,
        _ => 10,
    }
}

/// 週期起點:台北時間(UTC+8)本週一 00:00 / 本月 1 日 00:00,轉回 UTC 給查詢用
pub(super) fn period_start(now: DateTime<Utc>, period: LeaderboardPeriod) -> DateTime<Utc> {
    let tz = crate::utils::date::taipei_offset();
    let today = now.with_timezone(&tz).date_naive();
    let start = match period {
        LeaderboardPeriod::Weekly => {
            today - Duration::days(i64::from(today.weekday().num_days_from_monday()))
        }
        LeaderboardPeriod::Monthly => today.with_day(1).expect("每月必有 1 日"),
        // 總榜:早於任何一局的起點即可(vocab 功能 2026-07 才上線)
        LeaderboardPeriod::All => NaiveDate::from_ymd_opt(1970, 1, 1).expect("1970-01-01 合法"),
    };
    tz.from_local_datetime(&start.and_hms_opt(0, 0, 0).expect("00:00:00 合法"))
        .single()
        .expect("固定偏移無 DST,本地時間唯一")
        .with_timezone(&Utc)
}

/// 從「有玩過的台北日」清單算連續天數(純函式,可測)
///
/// `days` 需為去重且新到舊排序。今天還沒玩但昨天玩了 → 連續紀錄仍算延續
/// (今天內補打即可保住),中間斷一天以上就歸零。
pub fn streak_from_days(days: &[NaiveDate], today: NaiveDate) -> i32 {
    let Some(&latest) = days.first() else {
        return 0;
    };
    // 最後一次遊玩早於昨天 → 連續已斷
    let gap = (today - latest).num_days();
    if gap > 1 {
        return 0;
    }
    let mut streak = 1;
    let mut cursor = latest;
    for &day in &days[1..] {
        if (cursor - day).num_days() == 1 {
            streak += 1;
            cursor = day;
        } else {
            break;
        }
    }
    streak
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_curve_is_monotonic() {
        assert_eq!(exp_for_level(1), 0);
        assert_eq!(exp_for_level(2), 100);
        for level in 2..=50 {
            assert!(exp_for_level(level) > exp_for_level(level - 1));
        }
    }

    #[test]
    fn level_for_exp_matches_thresholds() {
        assert_eq!(level_for_exp(0), 1);
        assert_eq!(level_for_exp(99), 1);
        assert_eq!(level_for_exp(100), 2);
        for level in 1..=30 {
            let threshold = exp_for_level(level);
            assert_eq!(level_for_exp(threshold), level);
            if threshold > 0 {
                assert_eq!(level_for_exp(threshold - 1), level - 1);
            }
        }
    }

    #[test]
    fn answer_exp_scales_with_difficulty_combo_and_kind() {
        // 難度 1、首題答對(combo 1):10 + 2
        assert_eq!(answer_exp(1, 1, QuestionKind::Choice), 12);
        // 難度 5:10 + 20 = 30,再加 combo
        assert_eq!(answer_exp(5, 1, QuestionKind::Choice), 32);
        // combo 加成封頂在 10 連對
        assert_eq!(
            answer_exp(1, 10, QuestionKind::Choice),
            answer_exp(1, 99, QuestionKind::Choice)
        );
        // 拼字題 ×1.5
        assert_eq!(answer_exp(1, 1, QuestionKind::Spelling), 18);
    }

    #[test]
    fn difficulty_window_ramps_up() {
        assert_eq!(difficulty_window(0), (1, 2));
        assert_eq!(difficulty_window(10), (1, 3));
        assert_eq!(difficulty_window(25), (2, 4));
        assert_eq!(difficulty_window(100), (2, 5));
    }

    #[test]
    fn clamped_window_respects_language_bounds() {
        // 日文只 seed N5+N4(難度 1–2):中後段窗口不能整個落在題庫外
        assert_eq!(clamped_window(0, 1, 2), (1, 2));
        assert_eq!(clamped_window(25, 1, 2), (2, 2));
        assert_eq!(clamped_window(100, 1, 2), (2, 2));
        // 英文全分布(1–5):clamp 後行為不變
        assert_eq!(clamped_window(0, 1, 5), (1, 2));
        assert_eq!(clamped_window(100, 1, 5), (2, 5));
    }

    #[test]
    fn period_start_uses_taipei_boundaries() {
        // 2026-07-11(六)台北中午:週起點 = 台北 07-06(一)00:00 = UTC 07-05 16:00
        let now = Utc.with_ymd_and_hms(2026, 7, 11, 4, 0, 0).unwrap();
        assert_eq!(
            period_start(now, LeaderboardPeriod::Weekly),
            Utc.with_ymd_and_hms(2026, 7, 5, 16, 0, 0).unwrap()
        );
        // 月起點 = 台北 07-01 00:00 = UTC 06-30 16:00
        assert_eq!(
            period_start(now, LeaderboardPeriod::Monthly),
            Utc.with_ymd_and_hms(2026, 6, 30, 16, 0, 0).unwrap()
        );
    }

    #[test]
    fn period_start_respects_taipei_date_line() {
        // UTC 週日 20:00 = 台北週一 04:00,已跨進新的一週
        let now = Utc.with_ymd_and_hms(2026, 7, 5, 20, 0, 0).unwrap();
        assert_eq!(
            period_start(now, LeaderboardPeriod::Weekly),
            Utc.with_ymd_and_hms(2026, 7, 5, 16, 0, 0).unwrap()
        );
        // UTC 06-30 20:00 = 台北 07-01 04:00,月榜已換到 7 月
        let now = Utc.with_ymd_and_hms(2026, 6, 30, 20, 0, 0).unwrap();
        assert_eq!(
            period_start(now, LeaderboardPeriod::Monthly),
            Utc.with_ymd_and_hms(2026, 6, 30, 16, 0, 0).unwrap()
        );
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn streak_counts_consecutive_days_back_from_today() {
        let today = d(2026, 8, 21);
        let days = vec![d(2026, 8, 21), d(2026, 8, 20), d(2026, 8, 19)];
        assert_eq!(streak_from_days(&days, today), 3);
    }

    /// 今天還沒玩、昨天玩了 —— 連續紀錄還活著,不能顯示 0 逼使用者以為斷了
    #[test]
    fn streak_survives_a_day_not_yet_played() {
        let today = d(2026, 8, 21);
        let days = vec![d(2026, 8, 20), d(2026, 8, 19)];
        assert_eq!(streak_from_days(&days, today), 2);
    }

    #[test]
    fn streak_breaks_on_gap() {
        let today = d(2026, 8, 21);
        // 前天之後就沒玩 → 已斷
        assert_eq!(streak_from_days(&[d(2026, 8, 19)], today), 0);
        // 中間缺 08-19 → 只算回到 08-20
        let days = vec![d(2026, 8, 21), d(2026, 8, 20), d(2026, 8, 18)];
        assert_eq!(streak_from_days(&days, today), 2);
    }

    #[test]
    fn streak_of_no_history_is_zero() {
        assert_eq!(streak_from_days(&[], d(2026, 8, 21)), 0);
    }

    /// 總榜起點必須早於任何一局,否則總榜會漏資料
    #[test]
    fn period_start_all_predates_everything() {
        let now = Utc.with_ymd_and_hms(2026, 7, 11, 4, 0, 0).unwrap();
        assert!(
            period_start(now, LeaderboardPeriod::All)
                < Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn mask_sentence_hides_word_case_insensitive() {
        assert_eq!(
            mask_sentence("Apples are red.", "apple").as_deref(),
            Some("_____s are red.")
        );
        assert_eq!(
            mask_sentence("I like tea.", "coffee"),
            None
        );
    }
}
