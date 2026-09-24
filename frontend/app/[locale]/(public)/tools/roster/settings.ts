import { MAX_DAYS, MAX_NAMES, MAX_NAME_LEN, ROSTER_RULES, type RosterRule } from "@/libs/roster";

/**
 * 名單與參數存 localStorage：重新整理就要重打 20 個名字是這頁最大的日常痛點。
 * 版號在 key 裡，欄位改形狀時直接換號、不必寫遷移。
 */
export const STORAGE_KEY = "roster_settings_v1";

/** 會被持久化的表單狀態。整包一個 state：分成八個 useState 的話 mount 後要塞八次 */
export interface Settings {
    names: string[];
    days: string;
    rule: RosterRule;
    startDate: string;
    manualSlots: boolean;
    morningSlots: string;
    nightSlots: string;
    maxConsecutive: string;
}

export const DEFAULT_SETTINGS: Settings = {
    names: [],
    days: String(MAX_DAYS),
    rule: "fairness",
    startDate: "",
    manualSlots: false,
    morningSlots: "",
    nightSlots: "",
    maxConsecutive: "",
};

/** 逐欄驗型：手改過或舊版格式的 localStorage 不該讓整頁掛掉，對不上就退回預設 */
export function readSettings(): Settings | null {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return null;
        const saved = JSON.parse(raw) as Record<string, unknown>;
        const text = (value: unknown, fallback: string) => (typeof value === "string" ? value : fallback);
        return {
            names: Array.isArray(saved.names)
                ? saved.names
                      // 長度用字元數，跟後端的 chars 計數對齊
                      .filter((n): n is string => typeof n === "string" && [...n].length <= MAX_NAME_LEN)
                      .slice(0, MAX_NAMES)
                : DEFAULT_SETTINGS.names,
            days: text(saved.days, DEFAULT_SETTINGS.days),
            rule: ROSTER_RULES.includes(saved.rule as RosterRule) ? (saved.rule as RosterRule) : DEFAULT_SETTINGS.rule,
            startDate: text(saved.startDate, DEFAULT_SETTINGS.startDate),
            manualSlots: typeof saved.manualSlots === "boolean" ? saved.manualSlots : DEFAULT_SETTINGS.manualSlots,
            morningSlots: text(saved.morningSlots, DEFAULT_SETTINGS.morningSlots),
            nightSlots: text(saved.nightSlots, DEFAULT_SETTINGS.nightSlots),
            maxConsecutive: text(saved.maxConsecutive, DEFAULT_SETTINGS.maxConsecutive),
        };
    } catch {
        return null;
    }
}

/** 數字欄位一律存字串：存 number 的話清空欄位那一刻會被 `|| 1` 搶成 1，改不了值 */
export function clampNumber(raw: string, min: number, max: number, fallback: number): number {
    const parsed = parseInt(raw, 10);
    if (Number.isNaN(parsed)) return fallback;
    return Math.min(max, Math.max(min, parsed));
}

export function digitsOnly(value: string): string {
    return value.replace(/\D/g, "").slice(0, 3);
}
