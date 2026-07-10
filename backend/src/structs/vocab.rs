use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 題庫語言
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Ja,
}

impl Language {
    /// DB `words.language` / `vocab_runs.language` 用值
    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Ja => "ja",
        }
    }
}

/// 題庫單字(DB 對應)
#[derive(Clone, FromRow)]
pub struct Word {
    pub id: i64,
    pub word: String,
    pub part_of_speech: String,
    pub meaning_zh: String,
    pub example_sentence: String,
    pub difficulty: i16,
    /// 顯示用主讀音(平假名);英文為 None
    pub reading: Option<String>,
    /// 比對用全部合法讀音;None = 只接受 reading
    pub accepted_readings: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    Choice,
    Spelling,
}

/// 對局模式
/// - Survival:3 命,答錯扣命歸零結束
/// - Timed:限時,不限命,時間到結束
/// - TimedSurvival:限時 + 3 命,先到先算
/// - Review:只出答錯過的字,不計命/時間/經驗
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    #[default]
    Survival,
    Timed,
    TimedSurvival,
    Review,
}

impl RunMode {
    /// DB `vocab_runs.mode` 用值
    pub fn as_str(self) -> &'static str {
        match self {
            RunMode::Survival => "survival",
            RunMode::Timed => "timed",
            RunMode::TimedSurvival => "timed_survival",
            RunMode::Review => "review",
        }
    }
    pub fn has_lives(self) -> bool {
        matches!(self, RunMode::Survival | RunMode::TimedSurvival)
    }
    pub fn has_time(self) -> bool {
        matches!(self, RunMode::Timed | RunMode::TimedSurvival)
    }
}

/// 進行中對局的當前題目(只存 Redis;正解不下發前端)
#[derive(Serialize, Deserialize)]
pub struct CurrentQuestion {
    pub word_id: i64,
    pub kind: QuestionKind,
    pub difficulty: i16,
    /// 選擇題正解選項 index(spelling 為 None)
    pub answer_index: Option<usize>,
    /// 拼字題正解單字(choice 為 None);日文拼字題為顯示用主讀音
    pub answer_text: Option<String>,
    /// 日文拼字題:正規化後的全部合法讀音(比對用);英文留空走 ASCII 比對
    #[serde(default)]
    pub accepted_texts: Vec<String>,
    /// 該字讀音(答後回饋用;英文為 None)
    #[serde(default)]
    pub reading: Option<String>,
}

fn default_diff_min() -> i16 {
    1
}
fn default_diff_max() -> i16 {
    5
}

/// 進行中對局狀態(存 Redis,JSON 序列化)
#[derive(Serialize, Deserialize)]
pub struct RunState {
    /// 對局擁有者;訪客(未登入)為 None,不入 DB
    pub member_id: Option<i64>,
    #[serde(default)]
    pub mode: RunMode,
    /// 題庫語言(serde default 保證部署瞬間進行中的英文局照常)
    #[serde(default)]
    pub language: Language,
    /// 該語言題庫的難度上下界(開局查一次;難度窗口 clamp 用)
    #[serde(default = "default_diff_min")]
    pub diff_min: i16,
    #[serde(default = "default_diff_max")]
    pub diff_max: i16,
    pub lives: i32,
    pub combo: i32,
    pub max_combo: i32,
    pub answered: i32,
    pub correct: i32,
    pub exp: i64,
    pub started_at: DateTime<Utc>,
    /// 限時模式的截止時間(伺服器權威,非限時為 None)
    #[serde(default)]
    pub deadline: Option<DateTime<Utc>>,
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
    /// 題庫語言,缺省 en(舊 client 相容)
    #[serde(default)]
    pub language: Language,
    /// 限時模式時長(分鐘),接受 3 / 5 / 10,其他值一律回退 10
    pub duration_minutes: Option<i64>,
}

#[derive(Serialize)]
pub struct StartRunResponse {
    pub run_id: Uuid,
    pub mode: RunMode,
    pub language: Language,
    pub lives: i32,
    /// 複習模式的本局題數(其他模式為 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i32>,
    /// 限時模式的剩餘秒數(其他模式為 None),前端據此本地倒數
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_secs: Option<i64>,
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
    /// 該題單字讀音(日文局答後回饋;英文為 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading: Option<String>,
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
    /// 讀音(日文;英文為 None)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading: Option<String>,
    pub difficulty: i16,
    pub wrong_count: i32,
    pub correct_count: i32,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Serialize, FromRow)]
pub struct BestRun {
    pub mode: String,
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
    /// 各計分模式的最佳紀錄(每模式一筆,無紀錄的模式不出現)
    pub bests: Vec<BestRun>,
    pub total_runs: i64,
    pub words_learned: i64,
}
