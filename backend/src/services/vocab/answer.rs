use crate::{
    errors::{AppError, RequestError},
    repositories::{redis, vocab as vocab_repo},
    services::vocab_ja,
    state::AppState,
    structs::vocab::{AnswerRequest, AnswerResponse, QuestionKind, RunMode, RunResult},
};
use uuid::Uuid;
use super::engine::{answer_exp, level_for_exp};
use super::question::{next_question, pop_review_question};
use super::run::{finalize, finished_response, load_run, run_key, save_run};

pub async fn answer(
    state: &AppState,
    run_id: Uuid,
    caller: Option<i64>,
    req: &AnswerRequest,
) -> Result<AnswerResponse, AppError> {
    // 先擋長度再做任何事：日文讀音正規化會對整個輸入多趟掃描，而這支端點訪客可用
    req.validate()
        .map_err(crate::errors::RequestError::UnprocessableContent)?;

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
