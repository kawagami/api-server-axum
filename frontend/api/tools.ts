"use server";

import { fetchApi } from "@/libs/fetchApi";
import { clientIpHeaders } from "@/libs/client-ip";
import type { ConversionDirection } from "@/libs/convert-text";
import type { RosterEntry, RosterPlan, RosterRule, RosterWarning } from "@/libs/roster";

/** 後端只回 `converted_text`（原文是呼叫端自己傳的，回傳只會讓 response 體積翻倍） */
export async function postConvertText(text: string, direction: ConversionDirection): Promise<{ converted_text: string }> {
    return fetchApi(`${process.env.API_URL}/tools/convert_text`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...(await clientIpHeaders()) },
        body: JSON.stringify({ text, direction }),
    });
}

/**
 * 排班參數。`rule` 是 union 而不是 string —— 後端收的是 enum，打錯字會回 422，
 * 型別在這裡就要擋住。`morning_slots` / `night_slots` 兩者必須同時給或同時省略（後端驗證）。
 */
export interface RosterParams {
    names: string[];
    days: number;
    rule: RosterRule;
    morning_slots?: number;
    night_slots?: number;
    max_consecutive?: number;
}

/** 後端 `RosterResponse`。**不要回 `unknown[]` 讓呼叫端 cast** —— 那等於契約沒對齊 */
export interface RosterResponse {
    status: string;
    data: RosterEntry[];
    plan: RosterPlan;
    warnings: RosterWarning[];
}

export async function postRoster(params: RosterParams): Promise<RosterResponse> {
    return fetchApi(`${process.env.API_URL}/roster`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await clientIpHeaders()) },
        body: JSON.stringify(params),
        cache: 'no-store',
    });
}
