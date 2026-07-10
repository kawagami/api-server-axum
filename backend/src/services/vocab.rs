use crate::{
    errors::{AppError, RequestError},
    repositories::{redis, vocab as vocab_repo},
    services::vocab_ja,
    state::AppState,
    structs::vocab::{
        AnswerRequest, AnswerResponse, CurrentQuestion, Language, MistakeEntry, QuestionDto,
        QuestionKind, RunMode, RunResult, RunState, StartRunResponse, VocabMe, Word,
    },
};
use rand::Rng;
use uuid::Uuid;

const INITIAL_LIVES: i32 = 3;
/// 複習模式單局最多出幾個錯字
const REVIEW_BATCH: i64 = 20;
/// 進行中對局的 Redis TTL(秒),每次答題續期;放著不玩自動蒸發、不結算
const RUN_TTL_SECS: u64 = 1800;

fn run_key(run_id: Uuid) -> String {
    format!("vocab:run:{run_id}")
}

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
fn clamped_window(answered: i32, diff_min: i16, diff_max: i16) -> (i16, i16) {
    let (lo, hi) = difficulty_window(answered);
    (lo.clamp(diff_min, diff_max), hi.clamp(diff_min, diff_max))
}

/// 前 5 題全選擇題暖身,之後 30% 出拼字題
fn pick_kind(answered: i32) -> QuestionKind {
    if answered < 5 {
        QuestionKind::Choice
    } else if rand::thread_rng().gen_bool(0.3) {
        QuestionKind::Spelling
    } else {
        QuestionKind::Choice
    }
}

/// 把例句中的單字挖空(不分大小寫);找不到就不給例句
fn mask_sentence(sentence: &str, word: &str) -> Option<String> {
    let lower_sentence = sentence.to_lowercase();
    let lower_word = word.to_lowercase();
    let pos = lower_sentence.find(&lower_word)?;
    let mut masked = String::with_capacity(sentence.len());
    masked.push_str(&sentence[..pos]);
    masked.push_str(&"_".repeat(word.chars().count()));
    masked.push_str(&sentence[pos + lower_word.len()..]);
    Some(masked)
}

// ---------- 出題 ----------

async fn build_question(
    state: &AppState,
    run: &RunState,
    word: &Word,
    kind: QuestionKind,
) -> Result<(CurrentQuestion, QuestionDto), AppError> {
    let number = run.answered + 1;
    match kind {
        QuestionKind::Choice => {
            let distractors = vocab_repo::distractor_meanings(
                state.get_pool(),
                word.id,
                run.language.as_str(),
                word.difficulty,
                &word.meaning_zh,
            )
            .await?;
            // 干擾項不足(題庫太小)退回拼字題,不出殘缺選擇題
            if distractors.len() < 3 {
                return Box::pin(build_question(state, run, word, QuestionKind::Spelling)).await;
            }
            let answer_index = rand::thread_rng().gen_range(0..=distractors.len());
            let mut options = distractors;
            options.insert(answer_index, word.meaning_zh.clone());

            Ok((
                CurrentQuestion {
                    word_id: word.id,
                    kind,
                    difficulty: word.difficulty,
                    answer_index: Some(answer_index),
                    answer_text: None,
                    accepted_texts: vec![],
                    // 題面不下發 reading(未來讀音題型視其為正解),答後才回饋
                    reading: word.reading.clone(),
                },
                QuestionDto {
                    number,
                    kind,
                    difficulty: word.difficulty,
                    word: Some(word.word.clone()),
                    part_of_speech: Some(word.part_of_speech.clone()),
                    options: Some(options),
                    meaning_zh: None,
                    sentence_masked: None,
                    hint_first_letter: None,
                    hint_length: None,
                },
            ))
        }
        QuestionKind::Spelling => match run.language {
            Language::En => Ok((
                CurrentQuestion {
                    word_id: word.id,
                    kind,
                    difficulty: word.difficulty,
                    answer_index: None,
                    answer_text: Some(word.word.clone()),
                    accepted_texts: vec![],
                    reading: None,
                },
                QuestionDto {
                    number,
                    kind,
                    difficulty: word.difficulty,
                    word: None,
                    part_of_speech: Some(word.part_of_speech.clone()),
                    options: None,
                    meaning_zh: Some(word.meaning_zh.clone()),
                    sentence_masked: mask_sentence(&word.example_sentence, &word.word),
                    hint_first_letter: word.word.chars().next().map(|c| c.to_string()),
                    hint_length: Some(word.word.chars().count()),
                },
            )),
            // 日文拼字題 = 意思 → 讀音:輸入羅馬字/假名,轉平假名後比對;
            // 不做例句挖空(例句是變化形,洞的形狀對不上辭書形讀音)
            Language::Ja => {
                let reading = word.reading.clone().ok_or_else(|| {
                    AppError::RequestError(RequestError::UnprocessableContent(
                        "日文單字缺讀音,無法出拼字題".to_string(),
                    ))
                })?;
                let accepted_texts: Vec<String> = word
                    .accepted_readings
                    .clone()
                    .unwrap_or_else(|| vec![reading.clone()])
                    .iter()
                    .map(|r| vocab_ja::normalize_reading(r))
                    .collect();
                Ok((
                    CurrentQuestion {
                        word_id: word.id,
                        kind,
                        difficulty: word.difficulty,
                        answer_index: None,
                        answer_text: Some(reading.clone()),
                        accepted_texts,
                        reading: Some(reading.clone()),
                    },
                    QuestionDto {
                        number,
                        kind,
                        difficulty: word.difficulty,
                        word: None,
                        part_of_speech: Some(word.part_of_speech.clone()),
                        options: None,
                        meaning_zh: Some(word.meaning_zh.clone()),
                        sentence_masked: None,
                        // 提示用假名維度:首假名 + 拍數(玩家看的是轉換後的假名框)
                        hint_first_letter: reading.chars().next().map(|c| c.to_string()),
                        hint_length: Some(reading.chars().count()),
                    },
                ))
            }
        },
    }
}

async fn next_question(
    state: &AppState,
    run: &RunState,
) -> Result<(CurrentQuestion, QuestionDto), AppError> {
    let (min_d, max_d) = clamped_window(run.answered, run.diff_min, run.diff_max);
    let word = vocab_repo::random_word(
        state.get_pool(),
        run.member_id,
        run.language.as_str(),
        min_d,
        max_d,
        &run.seen_word_ids,
    )
    .await?
        .ok_or_else(|| {
            AppError::RequestError(RequestError::UnprocessableContent(
                "題庫沒有可出題的單字".to_string(),
            ))
        })?;
    build_question(state, run, &word, pick_kind(run.answered)).await
}

/// 複習模式:從佇列取下一個字出題;佇列空(或剩餘字都已下架)回 None
async fn pop_review_question(
    state: &AppState,
    run: &mut RunState,
) -> Result<Option<(CurrentQuestion, QuestionDto)>, AppError> {
    while !run.review_queue.is_empty() {
        let id = run.review_queue.remove(0);
        if let Some(word) = vocab_repo::word_by_id(state.get_pool(), id).await? {
            run.seen_word_ids.push(id);
            let q = build_question(state, run, &word, pick_kind(run.answered)).await?;
            return Ok(Some(q));
        }
    }
    Ok(None)
}

async fn save_run(state: &AppState, run_id: Uuid, run: &RunState) -> Result<(), AppError> {
    let json = serde_json::to_string(run)?;
    redis::cache_set(state.get_redis_pool(), &run_key(run_id), &json, RUN_TTL_SECS).await
}

async fn load_run(
    state: &AppState,
    run_id: Uuid,
    caller: Option<i64>,
) -> Result<RunState, AppError> {
    let json = redis::cache_get(state.get_redis_pool(), &run_key(run_id))
        .await?
        .ok_or(AppError::RequestError(RequestError::NotFound))?;
    let run: RunState = serde_json::from_str(&json)?;
    // member 的局只有本人能操作;訪客的局(None)憑 run_id 即可,不做擁有者檢查
    if let Some(owner) = run.member_id {
        if caller != Some(owner) {
            return Err(AppError::RequestError(RequestError::NotFound));
        }
    }
    Ok(run)
}

// ---------- 對外服務 ----------

/// 限時時長:只接受 3 / 5 / 10 分,其他一律回退 10
fn resolve_duration_minutes(m: Option<i64>) -> i64 {
    match m {
        Some(3) => 3,
        Some(5) => 5,
        _ => 10,
    }
}

pub async fn start_run(
    state: &AppState,
    member_id: Option<i64>,
    mode: RunMode,
    language: Language,
    duration_minutes: Option<i64>,
) -> Result<StartRunResponse, AppError> {
    // 複習模式需要已存的錯字紀錄,訪客不可用
    if mode == RunMode::Review && member_id.is_none() {
        return Err(AppError::AuthError(crate::errors::AuthError::Unauthorized));
    }

    // 該語言題庫的難度上下界(窗口 clamp 用);題庫為空直接擋下
    let (diff_min, diff_max) = vocab_repo::difficulty_bounds(state.get_pool(), language.as_str())
        .await?
        .ok_or_else(|| {
            AppError::RequestError(RequestError::UnprocessableContent(
                "題庫沒有可出題的單字".to_string(),
            ))
        })?;

    let now = chrono::Utc::now();
    let mut run = RunState {
        member_id,
        mode,
        language,
        diff_min,
        diff_max,
        lives: INITIAL_LIVES,
        combo: 0,
        max_combo: 0,
        answered: 0,
        correct: 0,
        exp: 0,
        started_at: now,
        deadline: None,
        seen_word_ids: vec![],
        review_queue: vec![],
        current: CurrentQuestion {
            word_id: 0,
            kind: QuestionKind::Choice,
            difficulty: 1,
            answer_index: None,
            answer_text: None,
            accepted_texts: vec![],
            reading: None,
        },
    };

    let mut remaining_secs = None;
    if mode.has_time() {
        let mins = resolve_duration_minutes(duration_minutes);
        run.deadline = Some(now + chrono::Duration::minutes(mins));
        remaining_secs = Some(mins * 60);
    }

    let (total, question) = match mode {
        RunMode::Review => {
            // 上方已保證 member 存在
            let mid = member_id.expect("review mode requires member");
            run.review_queue =
                vocab_repo::review_word_ids(state.get_pool(), mid, language.as_str(), REVIEW_BATCH)
                    .await?;
            let total = run.review_queue.len() as i32;
            let (current, question) = pop_review_question(state, &mut run)
                .await?
                .ok_or_else(|| {
                    AppError::RequestError(RequestError::UnprocessableContent(
                        "目前沒有需要複習的錯字".to_string(),
                    ))
                })?;
            run.current = current;
            (Some(total), question)
        }
        // 生存 / 限時 / 限時生存:都是隨機出題
        _ => {
            let (current, question) = next_question(state, &run).await?;
            run.seen_word_ids.push(current.word_id);
            run.current = current;
            (None, question)
        }
    };

    let run_id = Uuid::new_v4();
    save_run(state, run_id, &run).await?;

    Ok(StartRunResponse {
        run_id,
        mode,
        language,
        lives: run.lives,
        total,
        remaining_secs,
        question,
    })
}

/// 結算計分模式的對局:清 Redis、算新紀錄、落地、發經驗、回結算
async fn finalize(
    state: &AppState,
    run_id: Uuid,
    run: &RunState,
) -> Result<RunResult, AppError> {
    redis::cache_del(state.get_redis_pool(), &run_key(run_id)).await?;

    // 訪客:不入 DB,結算只回本局成績(經驗值當登入誘餌)
    let Some(mid) = run.member_id else {
        return Ok(RunResult {
            answered_count: run.answered,
            correct_count: run.correct,
            max_combo: run.max_combo,
            exp_gained: run.exp,
            total_exp: 0,
            level: 0,
            leveled_up: false,
            new_best: false,
            graduated: None,
        });
    };

    let previous_best =
        vocab_repo::best_run(state.get_pool(), mid, run.language.as_str(), run.mode.as_str())
            .await?;
    let new_best = previous_best.as_ref().is_none_or(|b| {
        run.correct > b.correct_count
            || (run.correct == b.correct_count && run.max_combo > b.max_combo)
    });

    vocab_repo::insert_run(state.get_pool(), run_id, mid, run).await?;
    let total_exp =
        vocab_repo::upsert_vocab_exp(state.get_pool(), mid, run.language.as_str(), run.exp)
            .await?;
    let level = level_for_exp(total_exp);
    let leveled_up = run.exp > 0 && level > level_for_exp(total_exp - run.exp);

    Ok(RunResult {
        answered_count: run.answered,
        correct_count: run.correct,
        max_combo: run.max_combo,
        exp_gained: run.exp,
        total_exp,
        level,
        leveled_up,
        new_best,
        graduated: None,
    })
}

/// 結算後的 AnswerResponse(答題已計入或棄置皆可用;feedback 欄由 caller 決定)
fn finished_response(
    run: &RunState,
    result: RunResult,
    correct: bool,
    correct_choice_index: Option<usize>,
    correct_text: Option<String>,
    reading: Option<String>,
    gained_exp: i64,
) -> AnswerResponse {
    AnswerResponse {
        correct,
        correct_choice_index,
        correct_text,
        reading,
        gained_exp,
        lives: run.lives,
        combo: run.combo,
        answered: run.answered,
        correct_count: run.correct,
        run_exp: run.exp,
        finished: true,
        question: None,
        result: Some(result),
    }
}

/// 限時到時或玩家主動結束:結算並回結果(限時模式專用)
pub async fn finish(
    state: &AppState,
    run_id: Uuid,
    caller: Option<i64>,
) -> Result<AnswerResponse, AppError> {
    let run = load_run(state, run_id, caller).await?;
    if !run.mode.has_time() {
        return Err(AppError::RequestError(RequestError::UnprocessableContent(
            "此模式不支援手動結束".to_string(),
        )));
    }
    let result = finalize(state, run_id, &run).await?;
    Ok(finished_response(&run, result, false, None, None, None, 0))
}

pub async fn answer(
    state: &AppState,
    run_id: Uuid,
    caller: Option<i64>,
    req: &AnswerRequest,
) -> Result<AnswerResponse, AppError> {
    let mut run = load_run(state, run_id, caller).await?;

    // 限時已到:棄置此題直接結算(正常由前端倒數歸零呼叫 finish,此為伺服器端安全網)
    if run.mode.has_time() {
        if let Some(dl) = run.deadline {
            if chrono::Utc::now() >= dl {
                let result = finalize(state, run_id, &run).await?;
                return Ok(finished_response(&run, result, false, None, None, None, 0));
            }
        }
    }

    let current = &run.current;

    let correct = match current.kind {
        QuestionKind::Choice => {
            let idx = req.choice_index.ok_or_else(|| {
                AppError::RequestError(RequestError::UnprocessableContent(
                    "選擇題須帶 choice_index".to_string(),
                ))
            })?;
            Some(idx) == current.answer_index
        }
        QuestionKind::Spelling => {
            let text = req.text.as_deref().ok_or_else(|| {
                AppError::RequestError(RequestError::UnprocessableContent(
                    "拼字題須帶 text".to_string(),
                ))
            })?;
            if current.accepted_texts.is_empty() {
                // 英文:忽略大小寫比對
                current
                    .answer_text
                    .as_deref()
                    .is_some_and(|a| a.eq_ignore_ascii_case(text.trim()))
            } else {
                // 日文:轉平假名後與任一合法讀音完全比對
                current
                    .accepted_texts
                    .contains(&vocab_ja::normalize_reading(text))
            }
        }
    };

    let correct_choice_index = current.answer_index;
    let correct_text = current.answer_text.clone();
    let question_reading = current.reading.clone();
    let question_word_id = current.word_id;
    let question_difficulty = current.difficulty;
    let question_kind = current.kind;

    run.answered += 1;

    // 學習進度:對錯都記(驅動錯題本與複習畢業判定);訪客不寫 DB
    if let Some(mid) = run.member_id {
        vocab_repo::upsert_word_stat(state.get_pool(), mid, question_word_id, correct).await?;
    }

    match run.mode {
        // 生存 / 限時 / 限時生存:隨機出題計分
        RunMode::Survival | RunMode::Timed | RunMode::TimedSurvival => {
            let gained_exp = if correct {
                run.correct += 1;
                run.combo += 1;
                run.max_combo = run.max_combo.max(run.combo);
                let gained = answer_exp(question_difficulty, run.combo, question_kind);
                run.exp += gained;
                gained
            } else {
                run.combo = 0;
                if run.mode.has_lives() {
                    run.lives -= 1;
                }
                0
            };

            // 有命模式命數歸零即結束(純限時模式靠時間到 / finish 結束)
            if run.mode.has_lives() && run.lives <= 0 {
                let result = finalize(state, run_id, &run).await?;
                return Ok(finished_response(
                    &run,
                    result,
                    correct,
                    correct_choice_index,
                    correct_text,
                    question_reading,
                    gained_exp,
                ));
            }

            let (current, question) = next_question(state, &run).await?;
            run.seen_word_ids.push(current.word_id);
            run.current = current;
            save_run(state, run_id, &run).await?;

            Ok(AnswerResponse {
                correct,
                correct_choice_index,
                correct_text,
                reading: question_reading,
                gained_exp,
                lives: run.lives,
                combo: run.combo,
                answered: run.answered,
                correct_count: run.correct,
                run_exp: run.exp,
                finished: false,
                question: Some(question),
                result: None,
            })
        }
        RunMode::Review => {
            // 複習不計命、不計 combo、不發經驗;答對只累加正確數(升級靠答對次數追上答錯次數)
            if correct {
                run.correct += 1;
            }

            match pop_review_question(state, &mut run).await? {
                Some((current, question)) => {
                    run.current = current;
                    save_run(state, run_id, &run).await?;
                    Ok(AnswerResponse {
                        correct,
                        correct_choice_index,
                        correct_text,
                        reading: question_reading,
                        gained_exp: 0,
                        lives: run.lives,
                        combo: 0,
                        answered: run.answered,
                        correct_count: run.correct,
                        run_exp: 0,
                        finished: false,
                        question: Some(question),
                        result: None,
                    })
                }
                None => {
                    redis::cache_del(state.get_redis_pool(), &run_key(run_id)).await?;
                    // 複習為 member-only,run.member_id 必為 Some
                    let mid = run.member_id.expect("review mode requires member");
                    let graduated =
                        vocab_repo::count_mastered_among(state.get_pool(), mid, &run.seen_word_ids)
                            .await? as i32;
                    let total_exp =
                        vocab_repo::vocab_exp(state.get_pool(), mid, run.language.as_str()).await?;

                    Ok(AnswerResponse {
                        correct,
                        correct_choice_index,
                        correct_text,
                        reading: question_reading,
                        gained_exp: 0,
                        lives: run.lives,
                        combo: 0,
                        answered: run.answered,
                        correct_count: run.correct,
                        run_exp: 0,
                        finished: true,
                        question: None,
                        result: Some(RunResult {
                            answered_count: run.answered,
                            correct_count: run.correct,
                            max_combo: 0,
                            exp_gained: 0,
                            total_exp,
                            level: level_for_exp(total_exp),
                            leveled_up: false,
                            new_best: false,
                            graduated: Some(graduated),
                        }),
                    })
                }
            }
        }
    }
}

pub async fn mistakes(
    state: &AppState,
    member_id: i64,
    language: Language,
) -> Result<Vec<MistakeEntry>, AppError> {
    vocab_repo::mistakes(state.get_pool(), member_id, language.as_str()).await
}

pub async fn me(
    state: &AppState,
    member_id: i64,
    language: Language,
) -> Result<VocabMe, AppError> {
    let lang = language.as_str();
    let exp = vocab_repo::vocab_exp(state.get_pool(), member_id, lang).await?;
    let level = level_for_exp(exp);
    let bests = vocab_repo::bests(state.get_pool(), member_id, lang).await?;
    let (total_runs, words_learned) =
        vocab_repo::member_stats(state.get_pool(), member_id, lang).await?;

    Ok(VocabMe {
        exp,
        level,
        level_exp: exp_for_level(level),
        next_level_exp: exp_for_level(level + 1),
        bests,
        total_runs,
        words_learned,
    })
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
