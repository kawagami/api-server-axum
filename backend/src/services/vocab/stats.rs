use crate::{
    errors::AppError,
    repositories::vocab as vocab_repo,
    state::AppState,
    structs::vocab::{
        Language, LeaderboardPeriod, LeaderboardResponse, MistakeListQuery, MistakesResponse, VocabMe,
    },
};
use chrono::Utc;
use super::engine::{exp_for_level, level_for_exp, period_start, streak_from_days};

pub async fn mistakes(
    state: &AppState,
    member_id: i64,
    q: &MistakeListQuery,
) -> Result<MistakesResponse, AppError> {
    let items = vocab_repo::mistakes(state.get_pool(), member_id, q).await?;
    let (total, reviewable) = vocab_repo::mistake_counts(state.get_pool(), member_id, q).await?;
    Ok(MistakesResponse {
        items,
        total,
        reviewable,
    })
}

/// 排行榜 top N 名額
const LEADERBOARD_SIZE: i64 = 20;

pub async fn leaderboard(
    state: &AppState,
    member_id: Option<i64>,
    language: Language,
    period: LeaderboardPeriod,
) -> Result<LeaderboardResponse, AppError> {
    let from = period_start(Utc::now(), period);
    let lang = language.as_str();
    let top = vocab_repo::leaderboard_top(state.get_pool(), lang, from, LEADERBOARD_SIZE).await?;
    let me = match member_id {
        Some(mid) => vocab_repo::leaderboard_me(state.get_pool(), lang, from, mid).await?,
        None => None,
    };
    Ok(LeaderboardResponse { top, me })
}

/// 連續天數往回看幾天就好 —— 再長的紀錄對「連續」沒有意義,查詢也不該無界
const STREAK_LOOKBACK_DAYS: i32 = 400;

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
    let days =
        vocab_repo::played_days(state.get_pool(), member_id, lang, STREAK_LOOKBACK_DAYS).await?;
    let today = crate::utils::date::taipei_today();

    Ok(VocabMe {
        exp,
        level,
        level_exp: exp_for_level(level),
        next_level_exp: exp_for_level(level + 1),
        bests,
        total_runs,
        words_learned,
        streak_days: streak_from_days(&days, today),
        played_today: days.first() == Some(&today),
    })
}
