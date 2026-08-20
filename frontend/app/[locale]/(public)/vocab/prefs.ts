// localStorage:遊玩偏好 + 訪客本機紀錄。
// 全部要在 mount 後才讀(SSR 沒有 localStorage,提前讀會 hydration 不一致)。

import type { VocabLanguage, VocabRunMode } from "@/types";

const PREFS_KEY = "vocab:prefs";
const GUEST_KEY = "vocab:guest";

export interface VocabPrefs {
    /** 限時模式時長(分鐘) */
    duration: number;
    /** 上次玩的模式,入口畫面標「上次」用 */
    lastMode: VocabRunMode | null;
}

/**
 * 訪客本機紀錄。
 *
 * 訪客局在後端完全不落地(member_id 為 None,不入 vocab_runs),所以這裡的數字
 * **只是本機統計**,不會、也不該補記回帳號 —— client 自報的 exp 無法驗證,
 * 補記等於開後門讓人刷排行榜。UI 必須明講「只存在這個瀏覽器」。
 */
export interface GuestStats {
    runs: number;
    bestCorrect: number;
    exp: number;
}

const EMPTY_GUEST: GuestStats = { runs: 0, bestCorrect: 0, exp: 0 };

function read<T>(key: string): Partial<T> | null {
    if (typeof window === "undefined") return null;
    try {
        const raw = window.localStorage.getItem(key);
        return raw ? (JSON.parse(raw) as Partial<T>) : null;
    } catch {
        return null; // 隱私模式 / 壞 JSON:當作沒偏好,不要讓遊戲進不去
    }
}

function write(key: string, value: unknown) {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.setItem(key, JSON.stringify(value));
    } catch {
        // 寫不進去(隱私模式 / 配額滿)就算了,偏好不是必要功能
    }
}

export function loadPrefs(fallbackDuration: number): VocabPrefs {
    const saved = read<VocabPrefs>(PREFS_KEY);
    return {
        duration: typeof saved?.duration === "number" ? saved.duration : fallbackDuration,
        lastMode: saved?.lastMode ?? null,
    };
}

export function savePrefs(prefs: VocabPrefs) {
    write(PREFS_KEY, prefs);
}

function guestKey(language: VocabLanguage) {
    return `${GUEST_KEY}:${language}`;
}

export function loadGuestStats(language: VocabLanguage): GuestStats {
    const saved = read<GuestStats>(guestKey(language));
    return {
        runs: saved?.runs ?? EMPTY_GUEST.runs,
        bestCorrect: saved?.bestCorrect ?? EMPTY_GUEST.bestCorrect,
        exp: saved?.exp ?? EMPTY_GUEST.exp,
    };
}

/** 累加一局訪客成績,回新的統計(呼叫端拿去 setState) */
export function addGuestRun(
    language: VocabLanguage,
    run: { correctCount: number; expGained: number },
): GuestStats {
    const prev = loadGuestStats(language);
    const next: GuestStats = {
        runs: prev.runs + 1,
        bestCorrect: Math.max(prev.bestCorrect, run.correctCount),
        exp: prev.exp + run.expGained,
    };
    write(guestKey(language), next);
    return next;
}
