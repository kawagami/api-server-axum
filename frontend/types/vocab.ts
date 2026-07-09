// 英文單字闖關(/member/vocab)

export type VocabQuestionKind = 'choice' | 'spelling';

export type VocabRunMode = 'survival' | 'timed' | 'timed_survival' | 'review';

export interface VocabQuestion {
    number: number; // 第幾題,1 起算
    kind: VocabQuestionKind;
    difficulty: number;
    // choice:英文單字選中文釋義
    word?: string;
    part_of_speech?: string;
    options?: string[];
    // spelling:中文釋義 + 挖空例句拼單字
    meaning_zh?: string;
    sentence_masked?: string;
    hint_first_letter?: string;
    hint_length?: number;
}

export interface VocabStartRun {
    run_id: string;
    mode: VocabRunMode;
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
    difficulty: number;
    wrong_count: number;
    correct_count: number;
    last_seen_at: string;
}

export interface VocabAnswer {
    correct: boolean;
    correct_choice_index?: number;
    correct_text?: string;
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

export interface VocabAnswerInput {
    choice_index?: number;
    text?: string;
}
