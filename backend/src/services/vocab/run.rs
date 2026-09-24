use crate::{
    errors::{AppError, RequestError},
    repositories::{redis, vocab as vocab_repo},
    state::AppState,
    structs::vocab::{
        AnswerResponse, CurrentQuestion,
        Language, QuestionKind, RunMode, RunResult, RunState, StartRunResponse,
    },
};
use uuid::Uuid;
use super::engine::{level_for_exp, resolve_duration_minutes};
use super::question::{next_question, pop_review_question};

const INITIAL_LIVES: i32 = 3;
/// 複習模式單局最多出幾個錯字
const REVIEW_BATCH: i64 = 20;
/// 進行中對局的 Redis TTL(秒),每次答題續期;放著不玩自動蒸發、不結算
const RUN_TTL_SECS: u64 = 1800;

pub(super) fn run_key(run_id: Uuid) -> String {
    format!("vocab:run:{run_id}")
}

pub(super) async fn save_run(state: &AppState, run_id: Uuid, run: &RunState) -> Result<(), AppError> {
    let json = serde_json::to_string(run)?;
    redis::cache_set(state.get_redis_pool(), &run_key(run_id), &json, RUN_TTL_SECS).await
}

pub(super) async fn load_run(
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

/// 結算計分模式的對局:算新紀錄、落地、發經驗、清 Redis、回結算
///
/// ⚠️ **Redis 的對局狀態必須最後才刪**。先刪再寫 DB 的話，落地失敗就再也重試不了
/// (正解與進度只存在 Redis)，成績與經驗直接消失。落地兩張表也必須同一個 transaction:
/// insert_run 成功但 upsert_vocab_exp 失敗會讓 vocab_runs 有紀錄(排行榜聚合看得到)
/// 而 member_vocab_exp 沒加,排行榜總和與玩家等級從此長期不一致。
pub(super) async fn finalize(
    state: &AppState,
    run_id: Uuid,
    run: &RunState,
) -> Result<RunResult, AppError> {
    // 訪客:不入 DB,結算只回本局成績(經驗值當登入誘餌)
    let Some(mid) = run.member_id else {
        redis::cache_del(state.get_redis_pool(), &run_key(run_id)).await?;
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

    let mut tx = state.get_pool().begin().await?;
    vocab_repo::insert_run_in_tx(&mut tx, run_id, mid, run).await?;
    let total_exp =
        vocab_repo::upsert_vocab_exp_in_tx(&mut tx, mid, run.language.as_str(), run.exp).await?;
    tx.commit().await?;

    // 成績確定落地後才清掉 Redis 的對局狀態(在此之前失敗都還能重試 —— 每條進 finalize
    // 的路徑都先 load_run,Redis 還在就重試得到)。殘留窗口:commit 成功但這行失敗時,
    // 重試會撞 vocab_runs 的 PK 衝突而報錯,但成績已經存好了,且 key 有 TTL 會自清。
    redis::cache_del(state.get_redis_pool(), &run_key(run_id)).await?;

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
pub(super) fn finished_response(
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
