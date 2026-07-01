"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";
import type { Setting, SettingsResponse } from "@/types";

export async function getSettings(): Promise<SettingsResponse> {
    return adminRequest<SettingsResponse>({ url: `${process.env.API_URL}/admin/settings` });
}

export async function updateSetting(key: string, value: string): Promise<Setting> {
    const response = await adminRequest<Setting>({
        url: `${process.env.API_URL}/admin/settings/${key}`,
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ value }),
    });
    revalidatePath("/admin/settings");
    return response;
}

/** 改全站主題：PATCH 後失效整站 layout cache，讓 getPublicSettings 立即重抓 */
export async function updateSiteTheme(theme: string): Promise<Setting> {
    const response = await updateSetting("site_theme", theme);
    revalidatePath("/", "layout");
    return response;
}

/** 改每日輪播對應表（星期→主題）。後端存 JSON 字串；改完同樣失效整站 layout */
export async function updateThemeRotation(rotation: Record<string, string>): Promise<Setting> {
    const response = await updateSetting("theme_rotation", JSON.stringify(rotation));
    revalidatePath("/", "layout");
    return response;
}
