//! 排班演算法（純函式、零 IO、零 state，可單測）。
//!
//! **為什麼不是舊版那樣**：舊版讓每位員工各自跑一份固定 pattern（`["早","晚","休"]`），
//! 起始位置用員工 index 當 offset。實測有三個實務缺陷：
//!
//! 1. **每日人力覆蓋是副作用而不是約束** —— `morning_heavy` 3 人時第 4 天是
//!    `{早:2, 休:1}`，**晚班 0 人**，店開不了。
//! 2. **人數 > pattern 長度就有人班表逐日相同** —— fairness 4 人時第 1 位與第 4 位
//!    永遠同進同出、同天休假。
//! 3. **休假天數不均且偏誤固定** —— morning_heavy 7 天時第 1 位早 4 休 1、第 2 位早 3 休 2，
//!    永遠是名單第一個人最累（`days` 不是 pattern 長度倍數時差距恆存在）。
//!
//! **現在的做法**：組一份**長度等於人數**的環狀 pattern，第 i 位在第 d 天的班別是
//! `pattern[(i + d) % 人數]`。這個構造的關鍵性質 —— 每一天所有人讀到的位置正好是
//! pattern 的一個排列，所以**每日各班人數恆等於 pattern 裡該班的張數**：覆蓋洞
//! 不再是「要檢查的性質」而是構造上不可能發生。同理每人跑滿一圈後班數完全相同，
//! 天數不滿一圈時差距也收在 1 天內（實測 1–31 天 × 1–100 人皆成立）。
//!
//! pattern 的排列方式負責兩條實務硬約束（舊版完全沒有）：
//! - **晚班不接隔日早班**：每段上班日內一律「早班在前、晚班在後」，段與段之間隔著休假，
//!   於是「晚→早」只可能出現在段內（不會，早在前）或跨段（中間有休）。
//! - **連續上班天數上限**：休假平均插進上班日之間（Bresenham），每段長度 ≤ ⌈上班日/休假數⌉。
//!
//! 人力不足到無法同時滿足時仍然排得出班表（覆蓋優先），但回 `RosterWarning` 讓前端
//! 明白告知，而不是靜默給一份看起來合法的爛班表。

use crate::structs::roster::{
    RosterPlan, RosterRequest, RosterRule, RosterWarning, StaffShift, DEFAULT_MAX_CONSECUTIVE,
    SHIFT_MORNING, SHIFT_NIGHT, SHIFT_OFF,
};
use std::collections::BTreeSet;

/// `rule` → 早／晚／休的人數權重。
fn weights(rule: RosterRule) -> [u32; 3] {
    match rule {
        RosterRule::Fairness => [1, 1, 1],
        RosterRule::MorningHeavy => [2, 1, 1],
        RosterRule::NightHeavy => [1, 2, 1],
    }
}

/// 把 `total` 拆成 `parts` 份，多出來的 1 用 Bresenham **平均散開**而不是全堆在前面。
///
/// 散開這件事有實際差別：休假段長 `[3,3,2,2]` 會讓 7 天視窗最多塞進 3 個休假（工時差 2 天），
/// `[3,2,3,2]` 則收在 1 天內。
fn split_even(total: u32, parts: u32) -> Vec<u32> {
    let base = total / parts;
    let extra = total % parts;
    (0..parts)
        .map(|i| base + ((i + 1) * extra / parts - i * extra / parts))
        .collect()
}

/// 決定每日各班人數。使用者有指定就照指定（已由 `validate` 保證 `m + n <= 人數`），
/// 沒指定就把人數依 `rule` 權重分三份（最大餘額法，同餘額按早→晚→休的固定順序，
/// 結果可重現）。
pub fn resolve_plan(req: &RosterRequest) -> RosterPlan {
    let people = req.names.len() as u32;
    let max_consecutive = req.max_consecutive.unwrap_or(DEFAULT_MAX_CONSECUTIVE);

    if let (Some(m), Some(n)) = (req.morning_slots, req.night_slots) {
        return RosterPlan {
            morning_slots: m,
            night_slots: n,
            rest_slots: people - m - n,
            max_consecutive,
        };
    }

    let w = weights(req.rule);
    let total_w: u32 = w.iter().sum();
    let mut share = [0u32; 3];
    let mut remainder = [0u32; 3];
    for i in 0..3 {
        share[i] = people * w[i] / total_w;
        remainder[i] = people * w[i] % total_w;
    }
    let mut left = people - share.iter().sum::<u32>();
    let mut order: Vec<usize> = (0..3).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(remainder[i]), i));
    for &i in &order {
        if left == 0 {
            break;
        }
        share[i] += 1;
        left -= 1;
    }

    let [mut morning, mut night, mut rest] = share;
    // 兩人以上就一定要兩班都有人：權重分下來可能給某班 0 人（人少時尤其），
    // 先從休假額度借，休假也沒了才從另一班借（那班至少留 1）。
    if people >= 2 {
        if morning == 0 {
            if rest > 0 {
                rest -= 1;
            } else {
                night -= 1;
            }
            morning += 1;
        }
        if night == 0 {
            if rest > 0 {
                rest -= 1;
            } else {
                morning -= 1;
            }
            night += 1;
        }
    }

    RosterPlan {
        morning_slots: morning,
        night_slots: night,
        rest_slots: rest,
        max_consecutive,
    }
}

/// pattern 是否有比整圈更短的週期。
///
/// 有短週期 = **有人班表逐日完全相同**（週期 p 的 pattern 會讓相隔 p 的兩個人同進同出），
/// 正是舊版被詬病的那件事，所以要偵測出來並打散。
fn shorter_period(pattern: &[&str]) -> Option<usize> {
    let len = pattern.len();
    (1..len)
        .filter(|&p| len.is_multiple_of(p))
        .find(|&p| (0..len).all(|i| pattern[i] == pattern[i % p]))
}

/// 依「每段上班日的早班張數」組出環狀 pattern：每段內早班在前、晚班在後，段後接休假。
fn emit(work_chunks: &[u32], morning_per_chunk: &[u32], rest_chunks: &[u32]) -> Vec<&'static str> {
    let mut pattern = Vec::new();
    for (i, &size) in work_chunks.iter().enumerate() {
        let morning = morning_per_chunk[i];
        pattern.extend(std::iter::repeat_n(SHIFT_MORNING, morning as usize));
        pattern.extend(std::iter::repeat_n(SHIFT_NIGHT, (size - morning) as usize));
        if let Some(&rests) = rest_chunks.get(i) {
            pattern.extend(std::iter::repeat_n(SHIFT_OFF, rests as usize));
        }
    }
    pattern
}

/// 組出長度等於人數的環狀 pattern（第 i 位第 d 天 = `pattern[(i + d) % 人數]`）。
pub fn build_pattern(plan: RosterPlan) -> Vec<&'static str> {
    let work = plan.morning_slots + plan.night_slots;
    let rests = plan.rest_slots;

    // 上班日切成幾段：休假數與上班日數取小（每段之間至少要有 1 天休才切得開）
    let chunk_count = if rests == 0 {
        1
    } else {
        rests.min(work).max(1)
    };
    let work_chunks = split_even(work, chunk_count);
    let rest_chunks = if rests == 0 {
        Vec::new()
    } else {
        split_even(rests, chunk_count)
    };

    // 早班在各段之間按段長比例分配（最大餘額法）
    let mut morning_per_chunk: Vec<u32> = work_chunks
        .iter()
        .map(|&size| plan.morning_slots * size / work)
        .collect();
    let mut remainder: Vec<u32> = work_chunks
        .iter()
        .map(|&size| plan.morning_slots * size % work)
        .collect();
    let mut left = plan.morning_slots - morning_per_chunk.iter().sum::<u32>();
    let mut order: Vec<usize> = (0..morning_per_chunk.len()).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(remainder[i]), i));
    for &i in &order {
        if left == 0 {
            break;
        }
        morning_per_chunk[i] += 1;
        remainder[i] = 0;
        left -= 1;
    }

    let mut pattern = emit(&work_chunks, &morning_per_chunk, &rest_chunks);

    // 各段組成完全相同時 pattern 會有短週期（例如 9 人 fairness → 早晚休 × 3），
    // 那等於三組人同進同出。把第一段與最後一段互換一個早／晚打散：總張數不變、
    // 段內仍是早在前，兩條硬約束都不受影響。
    let last = work_chunks.len() - 1;
    if last > 0 && shorter_period(&pattern).is_some() {
        let last_night = work_chunks[last] - morning_per_chunk[last];
        if morning_per_chunk[0] >= 1 && last_night >= 1 {
            morning_per_chunk[0] -= 1;
            morning_per_chunk[last] += 1;
            pattern = emit(&work_chunks, &morning_per_chunk, &rest_chunks);
        }
    }

    pattern
}

/// 依 `plan` 排出班表。回 `(每人班表, 警告碼)`。
///
/// 天數與人數皆已由 `RosterRequest::validate` 收在 31 / 100 以內，所以這裡是
/// O(人數 × 天數) ≤ 3100 格，成本可忽略。
pub fn build_roster(
    names: &[String],
    days: u32,
    plan: RosterPlan,
) -> (Vec<StaffShift>, Vec<RosterWarning>) {
    let people = names.len();
    let days = days as usize;
    let pattern = build_pattern(plan);

    let data: Vec<StaffShift> = names
        .iter()
        .enumerate()
        .map(|(i, name)| StaffShift {
            id: i + 1,
            name: name.clone(),
            shifts: (0..days)
                .map(|d| pattern[(i + d) % people].to_string())
                .collect(),
        })
        .collect();

    let mut warnings = BTreeSet::new();
    // 這兩個從 plan 就看得出來，跟排出來的結果無關
    if plan.rest_slots == 0 {
        warnings.insert(RosterWarning::Understaffed);
    }
    if plan.morning_slots == 0 || plan.night_slots == 0 {
        warnings.insert(RosterWarning::ShiftUncovered);
    }
    // 另外兩個掃**實際排出來的天數**而不是推論：天數不滿一圈時，環狀 pattern 上的
    // 違反有可能根本沒被排到，報了反而是假警報。
    for staff in &data {
        let mut streak = 0;
        for (d, shift) in staff.shifts.iter().enumerate() {
            if shift == SHIFT_OFF {
                streak = 0;
                continue;
            }
            streak += 1;
            if streak > plan.max_consecutive {
                warnings.insert(RosterWarning::MaxConsecutiveExceeded);
            }
            if d > 0 && staff.shifts[d - 1] == SHIFT_NIGHT && shift == SHIFT_MORNING {
                warnings.insert(RosterWarning::NightToMorning);
            }
        }
    }

    (data, warnings.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(people: usize, days: u32, rule: RosterRule) -> RosterRequest {
        RosterRequest {
            names: (0..people).map(|i| format!("p{i}")).collect(),
            days,
            rule,
            morning_slots: None,
            night_slots: None,
            max_consecutive: None,
        }
    }

    fn run(
        people: usize,
        days: u32,
        rule: RosterRule,
    ) -> (Vec<StaffShift>, Vec<RosterWarning>, RosterPlan) {
        let req = request(people, days, rule);
        assert!(req.validate().is_ok());
        let plan = resolve_plan(&req);
        let (data, warnings) = build_roster(&req.names, req.days, plan);
        (data, warnings, plan)
    }

    const RULES: [RosterRule; 3] = [
        RosterRule::Fairness,
        RosterRule::MorningHeavy,
        RosterRule::NightHeavy,
    ];

    fn count(staff: &StaffShift, shift: &str) -> usize {
        staff.shifts.iter().filter(|s| s.as_str() == shift).count()
    }

    /// pattern 長度必須等於人數，各班張數必須等於 plan —— 這是「每日覆蓋恆成立」的前提
    #[test]
    fn pattern_matches_plan_exactly() {
        for rule in RULES {
            for people in 1..=100 {
                let (_, _, plan) = run(people, 1, rule);
                let pattern = build_pattern(plan);
                assert_eq!(pattern.len(), people);
                let tally = |s: &str| pattern.iter().filter(|&&p| p == s).count();
                assert_eq!(tally(SHIFT_MORNING), plan.morning_slots as usize);
                assert_eq!(tally(SHIFT_NIGHT), plan.night_slots as usize);
                assert_eq!(tally(SHIFT_OFF), plan.rest_slots as usize);
            }
        }
    }

    /// 舊版最嚴重的缺陷：某天某班 0 人（morning_heavy 3 人的第 4 天晚班沒人）
    #[test]
    fn every_day_matches_the_plan() {
        for rule in RULES {
            for people in 1..=40 {
                for days in [1, 7, 14, 30, 31] {
                    let (data, _, plan) = run(people, days, rule);
                    for d in 0..days as usize {
                        let morning = data.iter().filter(|s| s.shifts[d] == SHIFT_MORNING).count();
                        let night = data.iter().filter(|s| s.shifts[d] == SHIFT_NIGHT).count();
                        assert_eq!(
                            (morning, night),
                            (plan.morning_slots as usize, plan.night_slots as usize),
                            "rule={rule:?} people={people} days={days} day={d} 人力與 plan 不符"
                        );
                    }
                }
            }
        }
    }

    /// 舊版 fairness 4 人時第 1 位與第 4 位班表逐日相同（永遠同進同出）
    #[test]
    fn no_two_people_share_an_identical_schedule() {
        for rule in RULES {
            for people in 2..=31 {
                let (data, _, _) = run(people, 31, rule);
                for a in 0..people {
                    for b in (a + 1)..people {
                        assert_ne!(
                            data[a].shifts, data[b].shifts,
                            "rule={rule:?} people={people}：{} 與 {} 班表完全相同",
                            data[a].name, data[b].name
                        );
                    }
                }
            }
        }
    }

    /// 舊版名單第一個人永遠最累。現在總工時差收在 1 天內。
    #[test]
    fn workload_is_even_within_one_day() {
        for rule in RULES {
            for people in 1..=100 {
                for days in 1..=31 {
                    let (data, _, _) = run(people, days, rule);
                    let work: Vec<usize> = data
                        .iter()
                        .map(|s| count(s, SHIFT_MORNING) + count(s, SHIFT_NIGHT))
                        .collect();
                    let spread = work.iter().max().unwrap() - work.iter().min().unwrap();
                    assert!(
                        spread <= 1,
                        "rule={rule:?} people={people} days={days} 工時差 {spread} 天：{work:?}"
                    );
                }
            }
        }
    }

    /// 早／晚班天數也要均衡（跑滿一圈完全相同；不滿一圈的差距實測上限 3 天）
    #[test]
    fn shift_mix_is_balanced() {
        for rule in RULES {
            for people in 2..=100 {
                let (data, _, _) = run(people, 31, rule);
                for shift in [SHIFT_MORNING, SHIFT_NIGHT] {
                    let per: Vec<usize> = data.iter().map(|s| count(s, shift)).collect();
                    let spread = per.iter().max().unwrap() - per.iter().min().unwrap();
                    assert!(
                        spread <= 3,
                        "rule={rule:?} people={people} {shift} 天數差 {spread}：{per:?}"
                    );
                }
                // 跑滿一圈（人數天）之後每個人的班數完全相同
                if people <= 31 {
                    let (full, _, _) = run(people, people as u32, rule);
                    let nights: Vec<usize> = full.iter().map(|s| count(s, SHIFT_NIGHT)).collect();
                    assert!(nights.iter().all(|&n| n == nights[0]));
                }
            }
        }
    }

    /// 人力足夠時不該有任何警告，兩條硬約束都要成立
    #[test]
    fn constraints_hold_when_staffed() {
        for rule in RULES {
            for people in 4..=100 {
                let (data, warnings, plan) = run(people, 31, rule);
                assert!(
                    warnings.is_empty(),
                    "rule={rule:?} people={people} 出現警告 {warnings:?}"
                );
                for staff in &data {
                    let mut streak = 0;
                    for (d, shift) in staff.shifts.iter().enumerate() {
                        if shift == SHIFT_OFF {
                            streak = 0;
                            continue;
                        }
                        streak += 1;
                        assert!(
                            streak <= plan.max_consecutive,
                            "{} 連上 {streak} 天（上限 {}）",
                            staff.name,
                            plan.max_consecutive
                        );
                        assert!(
                            !(d > 0
                                && staff.shifts[d - 1] == SHIFT_NIGHT
                                && shift == SHIFT_MORNING),
                            "{} 第 {d} 天晚班接早班",
                            staff.name
                        );
                    }
                }
            }
        }
    }

    /// 人力不足時仍要排得出班表，但必須明說。2 人 = 每天兩班都得有人 → 沒人休得到，
    /// 且晚班必然接早班。
    #[test]
    fn understaffed_still_returns_a_schedule_with_warnings() {
        let (data, warnings, plan) = run(2, 7, RosterRule::Fairness);
        assert_eq!(
            (plan.morning_slots, plan.night_slots, plan.rest_slots),
            (1, 1, 0)
        );
        assert!(warnings.contains(&RosterWarning::Understaffed));
        assert!(warnings.contains(&RosterWarning::NightToMorning));
        assert!(warnings.contains(&RosterWarning::MaxConsecutiveExceeded));
        for staff in &data {
            assert_eq!(staff.shifts.len(), 7);
            assert!(!staff.shifts.iter().any(|s| s == SHIFT_OFF));
        }
    }

    /// 一個人開不了兩班，要回 ShiftUncovered 而不是假裝排好了
    #[test]
    fn single_person_reports_uncovered_shift() {
        let (data, warnings, plan) = run(1, 5, RosterRule::Fairness);
        assert_eq!((plan.morning_slots, plan.night_slots), (1, 0));
        assert!(warnings.contains(&RosterWarning::ShiftUncovered));
        assert_eq!(data.len(), 1);
        assert_eq!(count(&data[0], SHIFT_MORNING), 5);
    }

    #[test]
    fn explicit_slots_win_over_rule_weights() {
        let mut req = request(10, 7, RosterRule::MorningHeavy);
        req.morning_slots = Some(2);
        req.night_slots = Some(3);
        req.max_consecutive = Some(3);
        assert!(req.validate().is_ok());
        let plan = resolve_plan(&req);
        assert_eq!(
            (
                plan.morning_slots,
                plan.night_slots,
                plan.rest_slots,
                plan.max_consecutive
            ),
            (2, 3, 5, 3)
        );
        let (data, warnings) = build_roster(&req.names, req.days, plan);
        assert!(warnings.is_empty());
        for d in 0..7 {
            assert_eq!(
                data.iter().filter(|s| s.shifts[d] == SHIFT_MORNING).count(),
                2
            );
            assert_eq!(
                data.iter().filter(|s| s.shifts[d] == SHIFT_NIGHT).count(),
                3
            );
        }
    }

    /// 休假比上班日還多時（手動把 slots 開很小），休假也要平均散開而不是全擠在尾端
    #[test]
    fn spreads_rest_when_rest_outnumbers_work() {
        let mut req = request(12, 31, RosterRule::Fairness);
        req.morning_slots = Some(2);
        req.night_slots = Some(1);
        assert!(req.validate().is_ok());
        let plan = resolve_plan(&req);
        assert_eq!(plan.rest_slots, 9);
        let (data, warnings) = build_roster(&req.names, req.days, plan);
        assert!(warnings.is_empty());
        let work: Vec<usize> = data.iter().map(|s| 31 - count(s, SHIFT_OFF)).collect();
        assert!(
            work.iter().max().unwrap() - work.iter().min().unwrap() <= 1,
            "{work:?}"
        );
    }

    /// max_consecutive 排不出來時要說，而不是靜默違反
    #[test]
    fn tight_max_consecutive_is_reported() {
        let mut req = request(10, 31, RosterRule::Fairness);
        req.max_consecutive = Some(1);
        let plan = resolve_plan(&req);
        // 10 人 fairness → 早 4 晚 3 休 3：7 個上班日切 3 段，最短的段也有 2 天
        let (_, warnings) = build_roster(&req.names, req.days, plan);
        assert!(warnings.contains(&RosterWarning::MaxConsecutiveExceeded));
    }

    /// morning_heavy / night_heavy 要真的偏，否則規則選單只是裝飾
    #[test]
    fn rule_weights_shift_the_balance() {
        let (_, _, fair) = run(12, 7, RosterRule::Fairness);
        let (_, _, morning) = run(12, 7, RosterRule::MorningHeavy);
        let (_, _, night) = run(12, 7, RosterRule::NightHeavy);
        assert_eq!(
            (fair.morning_slots, fair.night_slots, fair.rest_slots),
            (4, 4, 4)
        );
        assert!(morning.morning_slots > morning.night_slots);
        assert!(night.night_slots > night.morning_slots);
    }

    /// 全滿人力（100 人 × 31 天）不該爆掉，且每人班表長度正確
    #[test]
    fn handles_max_input() {
        let (data, _, _) = run(100, 31, RosterRule::NightHeavy);
        assert_eq!(data.len(), 100);
        assert!(data.iter().all(|s| s.shifts.len() == 31));
        assert_eq!(data[0].id, 1);
        assert_eq!(data[99].id, 100);
    }
}
