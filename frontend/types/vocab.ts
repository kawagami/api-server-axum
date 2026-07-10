// 單字闖關(/member/vocab;英文 en / 日文 ja 共用一套對局 API)

export type VocabQuestionKind = 'choice' | 'spelling';

export type VocabRunMode = 'survival' | 'timed' | 'timed_survival' | 'review';

export type VocabLanguage = 'en' | 'ja';

export interface VocabQuestion {
    number: number; // 第幾題,1 起算
    kind: VocabQuestionKind;
    difficulty: number;
    // choice:英文單字選中文釋義
    word?: string;
    part_of_speech?: string;
    options?: string[];
    // spelling:中文釋義 + 挖空例句拼單字
    // 日文語意:hint_first_letter = 首假名、hint_length = 假名拍數、無挖空例句
    meaning_zh?: string;
    sentence_masked?: string;
    hint_first_letter?: string;
    hint_length?: number;
}

export interface VocabStartRun {
    run_id: string;
    mode: VocabRunMode;
    language: VocabLanguage;
    lives: number;
    total?: number; // 複習模式的本局題數
    remaining_secs?: number; // 限時模式的剩餘秒數
    question: VocabQuestion;
}

export interface VocabRunResult {
    answered_count: number;
    correct_count: number;
    max_combo: number;
    exp_gained: number;
    total_exp: number;
    level: number;
    leveled_up: boolean;
    new_best: boolean;
    graduated?: number; // 複習模式:本局畢業(答對追上答錯)的字數
}

export interface VocabMistake {
    word: string;
    part_of_speech: string;
    meaning_zh: string;
    reading?: string; // 日文讀音;英文無
    difficulty: number;
    wrong_count: number;
    correct_count: number;
    last_seen_at: string;
}

export interface VocabAnswer {
    correct: boolean;
    correct_choice_index?: number;
    correct_text?: string;
    reading?: string; // 該題讀音(日文局答後回饋)
    gained_exp: number;
    lives: number;
    combo: number;
    answered: number;
    correct_count: number;
    run_exp: number;
    finished: boolean;
    question?: VocabQuestion;
    result?: VocabRunResult;
}

export interface VocabBestRun {
    mode: VocabRunMode;
    correct_count: number;
    max_combo: number;
    exp_gained: number;
}

export interface VocabMe {
    exp: number;
    level: number;
    level_exp: number;      // 本級起點累積 exp
    next_level_exp: number; // 升下一級所需累積 exp
    bests: VocabBestRun[];  // 各計分模式的最佳紀錄
    total_runs: number;
    words_learned: number;
}

// 排行榜週期(台北時間;weekly = 本週一起、monthly = 本月 1 日起)
export type VocabLeaderboardPeriod = 'weekly' | 'monthly';

export interface VocabLeaderboardRow {
    rank: number;
    name: string;
    avatar_url: string | null;
    exp: number;
    runs: number;
}

export interface VocabLeaderboard {
    top: VocabLeaderboardRow[];
    me?: { rank: number; exp: number }; // 登入且該週期有紀錄才有
}

export interface VocabAnswerInput {
    choice_index?: number;
    text?: string;
}

// ---------- 後台題庫管理(/admin/vocab) ----------

export interface AdminVocabWord {
    id: number;
    language: VocabLanguage;
    word: string;
    reading: string | null;
    accepted_readings: string[] | null;
    part_of_speech: string;
    meaning_zh: string;
    example_sentence: string;
    difficulty: number;
    enabled: boolean;
    wrong_total: number;
    correct_total: number;
}

/** 全欄位覆寫;表記與語言不可改 */
export interface UpdateVocabWordInput {
    reading: string | null;
    accepted_readings: string[] | null;
    part_of_speech: string;
    meaning_zh: string;
    example_sentence: string;
    difficulty: number;
    enabled: boolean;
}
