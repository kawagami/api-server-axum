"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";
import { runAction, type ActionResult } from "@/libs/api-error";
import type { Setting, SettingsResponse } from "@/types";

export async function getSettings(): Promise<SettingsResponse> {
    return adminRequest<SettingsResponse>({ url: `${process.env.API_URL}/admin/settings` });
}

// 以下寫入都由 client 元件直接呼叫，回結果而不 throw：後端對非法值回 422 並帶原因
// （「site_theme 必須是 …」「webauthn_rp_id 必須是 origin 的網域」），throw 出去在
// production 會被剝成通用訊息（見 libs/api-error.ts 的 ActionFailure）。

export async function updateSetting(key: string, value: string): Promise<ActionResult<Setting>> {
    return runAction(async () => {
        const response = await adminRequest<Setting>({
            url: `${process.env.API_URL}/admin/settings/${key}`,
            method: "PATCH",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ value }),
        });
        revalidatePath("/admin/settings");
        return response;
    });
}

/**
 * 一次更新多個 key（後端同 transaction、全過才寫）。
 * 用在「互相約束的設定組」—— 逐 key PATCH 的中間狀態必然違反不變式，
 * 後端會擋（如 webauthn_rp_id / webauthn_rp_origin 整組換網域）。
 */
export async function updateSettings(values: Record<string, string>): Promise<ActionResult<Setting[]>> {
    return runAction(async () => {
        const response = await adminRequest<Setting[]>({
            url: `${process.env.API_URL}/admin/settings`,
            method: "PATCH",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ values }),
        });
        revalidatePath("/admin/settings");
        return response;
    });
}

/** 寫入一個影響全站渲染的 key，成功才失效整站 layout cache（讓 getPublicSettings 立即重抓） */
async function updateSiteWideSetting(key: string, value: string): Promise<ActionResult<Setting>> {
    const result = await updateSetting(key, value);
    if (result.ok) revalidatePath("/", "layout");
    return result;
}

/** 改全站主題 */
export async function updateSiteTheme(theme: string): Promise<ActionResult<Setting>> {
    return updateSiteWideSetting("site_theme", theme);
}

/** 改每日輪播對應表（星期→主題）。後端存 JSON 字串 */
export async function updateThemeRotation(rotation: Record<string, string>): Promise<ActionResult<Setting>> {
    return updateSiteWideSetting("theme_rotation", JSON.stringify(rotation));
}

/** 改首頁功能卡片（顯示+排序，JSON 字串陣列） */
export async function updateHomeFeatures(keys: string[]): Promise<ActionResult<Setting>> {
    return updateSiteWideSetting("home_features", JSON.stringify(keys));
}

/** 改 instance 功能開關（"all" 或 JSON 字串陣列）。影響 API 路由/排程與全站導航 */
export async function updateEnabledFeatures(value: string): Promise<ActionResult<Setting>> {
    return updateSiteWideSetting("enabled_features", value);
}
