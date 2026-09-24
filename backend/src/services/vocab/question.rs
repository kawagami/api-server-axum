use crate::{
    errors::{AppError, RequestError},
    repositories::vocab as vocab_repo,
    services::vocab_ja,
    state::AppState,
    structs::vocab::{CurrentQuestion, Language, QuestionDto, QuestionKind, RunState, Word},
};
use rand::Rng;
use super::engine::{clamped_window, mask_sentence, pick_kind};

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
            let answer_index = rand::rng().random_range(0..=distractors.len());
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

pub(super) async fn next_question(
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
pub(super) async fn pop_review_question(
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
