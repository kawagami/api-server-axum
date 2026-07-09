use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 題庫單字(DB 對應)
#[derive(Clone, FromRow)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub part_of_speech: String,
    pub meaning_zh: String,
    pub example_sentence: String,
    pub difficulty: i16,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    Choice,
    Spelling,
}

/// 對局模式:生存(隨機出題賺經驗)/ 複習(只出答錯過的字,不計經驗)
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    #[default]
    Survival,
    Review,
}

/// 進行中對局的當前題目(只存 Redis;正解不下發前端)
#[derive(Serialize, Deserialize)]
pub struct CurrentQuestion {
    pub word_id: i64,
    pub kind: QuestionKind,
    pub difficulty: i16,
    /// 選擇題正解選項 index(spelling 為 None)
    pub answer_index: Option<usize>,
    /// 拼字題正解單字(choice 為 None)
    pub answer_text: Option<String>,
}

/// 進行中對局狀態(存 Redis,JSON 序列化)
#[derive(Serialize, Deserialize)]
pub struct RunState {
    pub member_id: i64,
    #[serde(default)]
    pub mode: RunMode,
    pub lives: i32,
    pub combo: i32,
    pub max_combo: i32,
    pub answered: i32,
    pub correct: i32,
    pub exp: i64,
    pub started_at: DateTime<Utc>,
    pub seen_word_ids: Vec<i64>,
    /// 複習模式待出題的 word_id 佇列(生存模式為空)
    #[serde(default)]
    pub review_queue: Vec<i64>,
    pub current: CurrentQuestion,
}

/// 下發前端的題目(不含正解)
#[derive(Serialize)]
pub struct QuestionDto {
    pub number: i32, // 第幾題,1 起算
    pub kind: QuestionKind,
    pub difficulty: i16,
    // choice:顯示英文單字,四選一中文釋義
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of_speech: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
    // spelling:顯示中文釋義 + 挖空例句,輸入拼字
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meaning_zh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentence_masked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_first_letter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint_length: Option<usize>,
}

#[derive(Deserialize)]
pub struct AnswerRequest {
    /// 選擇題:選項 index
    pub choice_index: Option<usize>,
    /// 拼字題:輸入的單字
    pub text: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct StartRunRequest {
    #[serde(default)]
    pub mode: RunMode,
}

#[derive(Serialize)]
pub struct StartRunResponse {
    pub run_id: Uuid,
    pub mode: RunMode,
    pub lives: i32,
    /// 複習模式的本局題數(生存模式為 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i32>,
    pub question: QuestionDto,
}

#[derive(Serialize)]
pub struct AnswerResponse {
    pub correct: bool,
    /// 選擇題正解選項 index(答對也回,前端標綠用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_choice_index: Option<usize>,
    /// 拼字題正解單字
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_text: Option<String>,
    pub gained_exp: i64,
    pub lives: i32,
    pub combo: i32,
    pub answered: i32,
    pub correct_count: i32,
    pub run_exp: i64,
    pub finished: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<QuestionDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<RunResult>,
}

/// 對局結算
#[derive(Serialize)]
pub struct RunResult {
    pub answered_count: i32,
    pub correct_count: i32,
    pub max_combo: i32,
    pub exp_gained: i64,
    pub total_exp: i64,
    pub level: i32,
    pub leveled_up: bool,
    pub new_best: bool,
    /// 複習模式:本局複習的字中,現在已「畢業」(答對次數追上答錯次數)的數量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graduated: Option<i32>,
}

/// 錯題本一列
#[derive(Serialize, FromRow)]
pub struct MistakeEntry {
    pub word: String,
    pub part_of_speech: String,
    pub meaning_zh: String,
    pub difficulty: i16,
    pub wrong_count: i32,
    pub correct_count: i32,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Serialize, FromRow)]
pub struct BestRun {
    pub correct_count: i32,
    pub max_combo: i32,
    pub exp_gained: i64,
}

/// GET /member/vocab/me 回傳
#[derive(Serialize)]
pub struct VocabMe {
    pub exp: i64,
    pub level: i32,
    /// 本級起點累積 exp
    pub level_exp: i64,
    /// 升下一級所需累積 exp
    pub next_level_exp: i64,
    pub best: Option<BestRun>,
    pub total_runs: i64,
    pub words_learned: i64,
}
