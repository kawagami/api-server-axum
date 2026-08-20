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

/// 拼字題答案的長度上限（bytes）。一個單字／讀音遠遠用不到這麼多。
///
/// 需要上限的理由：日文分支會對輸入跑 `vocab_ja::normalize_reading`
/// （NFKC → filter → to_lowercase → to_hiragana，多趟掃描 + 多份配置），
/// 而 vocab 對訪客開放，不擋的話 10MB 輸入會佔住 tokio worker。
/// 有了上限，成本可忽略，就不必為它包 spawn_blocking。
pub const MAX_ANSWER_TEXT_BYTES: usize = 256;

#[derive(Deserialize)]
pub struct AnswerRequest {
    /// 選擇題:選項 index
    pub choice_index: Option<usize>,
    /// 拼字題:輸入的單字
    pub text: Option<String>,
}

impl AnswerRequest {
    /// 純函式驗證，可測。
    pub fn validate(&self) -> Result<(), String> {
        if let Some(t) = &self.text {
            if t.len() > MAX_ANSWER_TEXT_BYTES {
                return Err(format!("text 長度上限 {MAX_ANSWER_TEXT_BYTES} bytes"));
            }
        }
        Ok(())
    }
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

/// 錯題本一頁上限;錯題本會隨學習無界成長,端點必須有上限
pub const MISTAKES_MAX_LIMIT: i64 = 100;
pub const MISTAKES_DEFAULT_LIMIT: i64 = 50;

/// 錯題本排序
#[derive(Deserialize, Clone, Copy, PartialEq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum MistakeSort {
    /// 未掌握優先、錯最多優先(預設)
    #[default]
    Wrong,
    /// 最近答過優先
    Recent,
    /// 難度高優先
    Difficulty,
    /// 表記字母序
    Word,
}

impl MistakeSort {
    /// ORDER BY 片段。回固定字串,不含使用者輸入,可安全內插。
    pub fn order_by(self) -> &'static str {
        match self {
            MistakeSort::Wrong => {
                "(s.correct_count >= s.wrong_count), s.wrong_count DESC, s.last_seen_at DESC"
            }
            MistakeSort::Recent => "s.last_seen_at DESC, s.wrong_count DESC",
            MistakeSort::Difficulty => "w.difficulty DESC, s.wrong_count DESC, w.word",
            MistakeSort::Word => "w.word, s.wrong_count DESC",
        }
    }
}

/// GET /member/vocab/mistakes 的 query
#[derive(Deserialize, Default)]
pub struct MistakeListQuery {
    #[serde(default)]
    pub language: Language,
    /// 表記 / 讀音 / 釋義模糊搜尋
    pub q: Option<String>,
    #[serde(default)]
    pub sort: MistakeSort,
    /// 只看未掌握(答錯 > 答對)
    #[serde(default)]
    pub unmastered: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl MistakeListQuery {
    /// 空字串搜尋等於沒搜尋(前端清空輸入框會送 `q=`)
    pub fn search(&self) -> Option<&str> {
        self.q.as_deref().map(str::trim).filter(|s| !s.is_empty())
    }
    pub fn limit(&self) -> i64 {
        self.limit
            .unwrap_or(MISTAKES_DEFAULT_LIMIT)
            .clamp(1, MISTAKES_MAX_LIMIT)
    }
    pub fn offset(&self) -> i64 {
        self.offset.unwrap_or(0).max(0)
    }
}

/// GET /member/vocab/mistakes 回傳
///
/// `total` 跟著搜尋/篩選條件走(分頁用),`reviewable` 一律是全部未掌握字數
/// —— 複習按鈕的數字不能被錯題本的搜尋條件影響。
#[derive(Serialize)]
pub struct MistakesResponse {
    pub items: Vec<MistakeEntry>,
    pub total: i64,
    pub reviewable: i64,
}

// ---------- 後台題庫管理 ----------

/// 後台題庫列表一列(含全會員答題統計,揪出高錯誤率的爛字用)
#[derive(Serialize, FromRow)]
pub struct AdminWord {
    pub id: i64,
    pub language: String,
    pub word: String,
    pub reading: Option<String>,
    pub accepted_readings: Option<Vec<String>>,
    pub part_of_speech: String,
    pub meaning_zh: String,
    pub example_sentence: String,
    pub difficulty: i16,
    pub enabled: bool,
    pub wrong_total: i64,
    pub correct_total: i64,
}

#[derive(Deserialize)]
pub struct AdminWordListQuery {
    pub language: Option<String>,
    pub difficulty: Option<i16>,
    pub enabled: Option<bool>,
    /// 表記 / 讀音 / 釋義模糊搜尋
    pub q: Option<String>,
    /// `wrong` = 錯最多優先(預設依 id)
    pub sort: Option<String>,
}

#[derive(Serialize)]
pub struct AdminWordListResponse {
    pub data: Vec<AdminWord>,
    pub total: i64,
}

/// 全欄位覆寫更新;表記與語言不可改(改表記等於換字,走 seed 流程)
#[derive(Deserialize)]
pub struct UpdateWordRequest {
    pub reading: Option<String>,
    pub accepted_readings: Option<Vec<String>>,
    pub part_of_speech: String,
    pub meaning_zh: String,
    pub example_sentence: String,
    pub difficulty: i16,
    pub enabled: bool,
}

#[derive(Serialize, FromRow)]
pub struct BestRun {
    pub mode: String,
    pub correct_count: i32,
    pub max_combo: i32,
    pub exp_gained: i64,
}

/// 排行榜週期(台北時間;weekly = 本週一起、monthly = 本月 1 日起、all = 不限期間)
#[derive(Deserialize, Clone, Copy, PartialEq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LeaderboardPeriod {
    #[default]
    Weekly,
    Monthly,
    All,
}

/// 排行榜一列(top N;name/avatar 取自 members,公開顯示)
#[derive(Serialize, FromRow)]
pub struct LeaderboardRow {
    pub rank: i64,
    pub name: String,
    pub avatar_url: Option<String>,
    pub exp: i64,
    pub runs: i64,
}

/// 自己的名次(登入且該週期有紀錄才有)
#[derive(Serialize, FromRow)]
pub struct LeaderboardMe {
    pub rank: i64,
    pub exp: i64,
}

/// GET /member/vocab/leaderboard 回傳
#[derive(Serialize)]
pub struct LeaderboardResponse {
    pub top: Vec<LeaderboardRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub me: Option<LeaderboardMe>,
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
    /// 連續遊玩天數(台北時間;今天還沒玩但昨天玩了仍算延續)
    pub streak_days: i32,
    /// 今天(台北時間)是否已玩過 —— 前端提示「今天還沒玩,連續紀錄會斷」用
    pub played_today: bool,
}

#[cfg(test)]
mod answer_request_tests {
    use super::*;

    fn with_text(t: Option<&str>) -> AnswerRequest {
        AnswerRequest {
            choice_index: None,
            text: t.map(str::to_string),
        }
    }

    #[test]
    fn accepts_normal_answer() {
        assert!(with_text(Some("apple")).validate().is_ok());
        assert!(with_text(Some("とうきょう")).validate().is_ok());
    }

    /// 選擇題不帶 text，不該被誤擋
    #[test]
    fn accepts_missing_text() {
        assert!(with_text(None).validate().is_ok());
    }

    #[test]
    fn accepts_exactly_at_limit() {
        let s = "a".repeat(MAX_ANSWER_TEXT_BYTES);
        assert!(with_text(Some(s.as_str())).validate().is_ok());
    }

    /// 這條守的是「訪客可觸發的 CPU 阻塞」：日文讀音正規化會對整個輸入多趟掃描
    #[test]
    fn rejects_oversized_answer() {
        let s = "a".repeat(MAX_ANSWER_TEXT_BYTES + 1);
        assert!(with_text(Some(s.as_str())).validate().is_err());
        let huge = "a".repeat(10 * 1024 * 1024);
        assert!(with_text(Some(huge.as_str())).validate().is_err());
    }
}
