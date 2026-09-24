import type { VocabMistakeSort, VocabQuestion, VocabRunMode, VocabRunResult } from "@/types";

export type Phase = "idle" | "playing" | "finished";

export interface Feedback {
    correct: boolean;
    selectedIndex: number | null;
    correctChoiceIndex: number | null;
    correctText: string | null;
    reading: string | null; // 日文局答後回饋的讀音
    gainedExp: number;
}

/** 回饋期間先攔住的下一步;倒數到了或使用者主動跳過才套用 */
export interface Pending {
    finished: boolean;
    leveledUp: boolean;
    result: VocabRunResult | null;
    question: VocabQuestion | null;
}

export interface MistakeQuery {
    q: string;
    sort: VocabMistakeSort;
    unmastered: boolean;
}

export const DEFAULT_MISTAKE_QUERY: MistakeQuery = { q: "", sort: "wrong", unmastered: false };

export function hasLives(mode: VocabRunMode) {
    return mode === "survival" || mode === "timed_survival";
}
export function hasTimer(mode: VocabRunMode) {
    return mode === "timed" || mode === "timed_survival";
}
