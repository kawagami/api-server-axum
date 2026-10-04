"use server";

import memberRequest from "@/libs/memberRequest";
import { runAction, type ActionResult } from "@/libs/api-error";
import type { FoodLogDay, FoodLogEntry, FoodLogEntryInput, FoodLogSuggestion, FoodLogSummary } from "@/types";

const BASE = () => `${process.env.API_URL}/member/food_log`;

export interface FoodLogOverview {
    days: FoodLogDay[];
    suggestions: FoodLogSuggestion[];
    summary: FoodLogSummary;
}

/**
 * 頁面一次要的三份資料。包成一支是因為 client 端寫入後要整份重抓，而 Next 一次只送
 * 一個 Server Action（三支分開呼叫會排隊成三趟），這裡在 server 端併發打後端。
 * `from` / `to` 為 YYYY-MM-DD（台北日），兩端皆含。
 */
export async function getFoodLogOverview(from: string, to: string): Promise<FoodLogOverview> {
    const query = new URLSearchParams({ from, to });
    const [days, suggestions, summary] = await Promise.all([
        memberRequest<FoodLogDay[]>({ url: `${BASE()}/days?${query}` }),
        memberRequest<FoodLogSuggestion[]>({ url: `${BASE()}/suggestions` }),
        memberRequest<FoodLogSummary>({ url: `${BASE()}/summary` }),
    ]);
    return {
        days: days ?? [],
        suggestions: suggestions ?? [],
        summary: summary ?? { today: 0, week: 0, month: 0 },
    };
}

/** 往前翻：只要時間軸，不重抓選項與合計 */
export async function getFoodLogDays(from: string, to: string): Promise<ActionResult<FoodLogDay[]>> {
    return runAction(async () => {
        const query = new URLSearchParams({ from, to });
        return (await memberRequest<FoodLogDay[]>({ url: `${BASE()}/days?${query}` })) ?? [];
    });
}

// 寫入一律順手回整份 overview：client 少一趟 Server Action（它們是排隊送的），
// 合計與「常吃」選項也跟著更新。

export async function postFoodLogEntry(
    input: FoodLogEntryInput,
    range: { from: string; to: string },
): Promise<ActionResult<FoodLogOverview>> {
    return runAction(async () => {
        await memberRequest<FoodLogEntry>({
            url: `${BASE()}/entries`,
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(input),
        });
        return getFoodLogOverview(range.from, range.to);
    });
}

export async function putFoodLogEntry(
    id: number,
    input: FoodLogEntryInput,
    range: { from: string; to: string },
): Promise<ActionResult<FoodLogOverview>> {
    return runAction(async () => {
        await memberRequest<FoodLogEntry>({
            url: `${BASE()}/entries/${id}`,
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(input),
        });
        return getFoodLogOverview(range.from, range.to);
    });
}

export async function deleteFoodLogEntry(
    id: number,
    range: { from: string; to: string },
): Promise<ActionResult<FoodLogOverview>> {
    return runAction(async () => {
        await memberRequest<null>({ url: `${BASE()}/entries/${id}`, method: "DELETE" });
        return getFoodLogOverview(range.from, range.to);
    });
}

/** 空白 = 刪掉當天備註 */
export async function putFoodLogDayNote(
    date: string,
    note: string,
    range: { from: string; to: string },
): Promise<ActionResult<FoodLogOverview>> {
    return runAction(async () => {
        await memberRequest<null>({
            url: `${BASE()}/days/${date}`,
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ note }),
        });
        return getFoodLogOverview(range.from, range.to);
    });
}
