import type { FoodLogSuggestion, Meal } from "@/types";

/** 一天之內的顯示順序，與後端 `Meal::ALL` 相同 */
export const MEALS: readonly Meal[] = ["breakfast", "lunch", "dinner", "snack", "late_night", "other"];

/** 台北日 YYYY-MM-DD（與後端 taipei_today() 對齊；en-CA 的輸出剛好是這個格式） */
export function taipeiToday(now: Date = new Date()): string {
    return new Intl.DateTimeFormat("en-CA", { timeZone: "Asia/Taipei" }).format(now);
}

/** YYYY-MM-DD 加減天數（用 UTC 算，避開本地時區的 DST） */
export function addDays(date: string, days: number): string {
    const d = new Date(`${date}T00:00:00Z`);
    d.setUTCDate(d.getUTCDate() + days);
    return d.toISOString().slice(0, 10);
}

export function minDate(a: string, b: string): string {
    return a < b ? a : b;
}

/** 依台北時間猜現在是哪一餐：補記別天的紀錄時再手動切 */
export function defaultMeal(now: Date = new Date()): Meal {
    const parts = new Intl.DateTimeFormat("en-GB", {
        timeZone: "Asia/Taipei",
        hour: "2-digit",
        minute: "2-digit",
        hourCycle: "h23",
    }).formatToParts(now);
    const hour = Number(parts.find(p => p.type === "hour")?.value ?? 12);
    const minute = Number(parts.find(p => p.type === "minute")?.value ?? 0);
    const t = hour * 60 + minute;
    if (t < 4 * 60) return "late_night";
    if (t < 10 * 60 + 30) return "breakfast";
    if (t < 14 * 60) return "lunch";
    if (t < 17 * 60) return "snack";
    if (t < 21 * 60) return "dinner";
    return "late_night";
}

const FREQUENT_LIMIT = 8;
const RECENT_LIMIT = 12;

/**
 * 「這餐常吃」= 在這個餐別吃過的，依次數多到少；「最近」= 其餘依最後吃的日期新到舊。
 * 後端已依最後日期排好，這裡只做餐別那一刀，切餐別不必重打 API。
 */
export function rankSuggestions(list: FoodLogSuggestion[], meal: Meal) {
    const frequent = list
        .filter(s => s.meal_counts[meal] > 0)
        .sort((a, b) => b.meal_counts[meal] - a.meal_counts[meal] || b.last_eaten_on.localeCompare(a.last_eaten_on))
        .slice(0, FREQUENT_LIMIT);
    const picked = new Set(frequent);
    const recent = list.filter(s => !picked.has(s)).slice(0, RECENT_LIMIT);
    return { frequent, recent };
}

/** 店家或品項含關鍵字（不分大小寫） */
export function searchSuggestions(list: FoodLogSuggestion[], query: string): FoodLogSuggestion[] {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return list.filter(s => `${s.store ?? ""} ${s.item}`.toLowerCase().includes(q));
}

/** 金額顯示：`$1,234`，`null`（點數兌換 / 請客）由呼叫端自己決定文案 */
export function formatAmount(n: number, locale: string): string {
    return `$${new Intl.NumberFormat(locale).format(n)}`;
}
