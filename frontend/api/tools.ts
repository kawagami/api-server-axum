"use server";

import { fetchApi } from "@/libs/fetchApi";

export async function getNewPassword(count = 1, length = 8): Promise<string[]> {
    return fetchApi(
        `${process.env.API_URL}/tools/new_password?count=${count}&length=${length}`,
        { cache: 'no-store' }
    );
}

export async function postConvertText(text: string, direction: "t2s" | "s2t"): Promise<{ original_text: string; converted_text: string }> {
    return fetchApi(`${process.env.API_URL}/tools/convert_text`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text, direction }),
    });
}

interface RosterParams {
    names: string[];
    days: number | string;
    rule: string;
}

interface RosterResponse {
    status: string;
    data: unknown[];
}

export async function postRoster(params: RosterParams): Promise<RosterResponse> {
    return fetchApi(`${process.env.API_URL}/roster`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            names: params.names,
            days: parseInt(String(params.days)),
            rule: params.rule,
        }),
        cache: 'no-store',
    });
}
