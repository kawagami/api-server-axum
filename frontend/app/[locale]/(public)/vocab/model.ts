import type { VocabMistakeSort, VocabQuestion, VocabRunMode, VocabRunResult } from "@/types";

export type Phase = "idle" | "playing" | "finished";

export interface Feedback {
    correct: boolean;
    selectedIndex: number | null;
    correctChoiceIndex: number | null;
    correctText: string | null;
    reading: string | null; // 日文局答後回饋的讀音
    gainedExp: number;
    answer: string | null;  // 拼字題使用者送出的字,答錯時拿來跟正解逐字比對
}

/** 回饋期間先攔住的下一步;倒數到了或使用者主動跳過才套用 */
export interface Pending {
    finished: boolean;
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
/** 連對里程碑:5、10、20,之後每 10 題 */
export function isComboMilestone(combo: number) {
    return combo === 5 || combo === 10 || (combo >= 20 && combo % 10 === 0);
}
export function hasTimer(mode: VocabRunMode) {
    return mode === "timed" || mode === "timed_survival";
}

/**
 * 拼字答錯時的逐字比對:用最長共同子序列(LCS)對齊兩串字,標出哪些字對得上。
 * 比逐位置比較好 —— 漏打或多打一個字時,後面的字不會因為錯位而全部被標錯。
 * 只用來顯示;對錯判定仍以後端為準(大小寫不分,跟後端一致)。
 */
export function diffMarks(answer: string, correct: string): { answer: boolean[]; correct: boolean[] } {
    const a = Array.from(answer.toLowerCase());
    const c = Array.from(correct.toLowerCase());
    const dp: number[][] = Array.from({ length: a.length + 1 }, () => new Array<number>(c.length + 1).fill(0));
    for (let i = a.length - 1; i >= 0; i--) {
        for (let j = c.length - 1; j >= 0; j--) {
            dp[i][j] = a[i] === c[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
        }
    }
    const am = new Array<boolean>(a.length).fill(false);
    const cm = new Array<boolean>(c.length).fill(false);
    let i = 0, j = 0;
    while (i < a.length && j < c.length) {
        if (a[i] === c[j]) { am[i] = true; cm[j] = true; i++; j++; }
        else if (dp[i + 1][j] >= dp[i][j + 1]) i++;
        else j++;
    }
    return { answer: am, correct: cm };
}
