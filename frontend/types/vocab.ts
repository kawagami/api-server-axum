// 英文單字闖關(/member/vocab)

export type VocabQuestionKind = 'choice' | 'spelling';

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
    lives: number;
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
    correct_count: number;
    max_combo: number;
    exp_gained: number;
}

export interface VocabMe {
    exp: number;
    level: number;
    level_exp: number;      // 本級起點累積 exp
    next_level_exp: number; // 升下一級所需累積 exp
    best: VocabBestRun | null;
    total_runs: number;
    words_learned: number;
}

export interface VocabAnswerInput {
    choice_index?: number;
    text?: string;
}
