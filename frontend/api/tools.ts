"use server";

import { fetchApi } from "@/libs/fetchApi";
import { clientIpHeaders } from "@/libs/client-ip";
import type { RosterEntry, RosterPlan, RosterRule, RosterWarning } from "@/libs/roster";

export async function getNewPassword(count = 1, length = 8): Promise<string[]> {
    return fetchApi(
        `${process.env.API_URL}/tools/new_password?count=${count}&length=${length}`,
        { cache: 'no-store', headers: await clientIpHeaders() }
    );
}

export async function postConvertText(text: string, direction: "t2s" | "s2t"): Promise<{ original_text: string; converted_text: string }> {
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
