"use server";

import { fetchApi } from "@/libs/fetchApi";

// 公開設定白名單（後端硬編碼 PUBLIC_KEYS）
export interface PublicSettings {
    site_theme?: string;          // 7 套主題之一 或 'auto'（每日輪播）
    default_color_mode?: string;  // light / dark / system
    theme_rotation?: unknown;     // 星期→主題對應表（物件或 JSON 字串），normalizeRotation 收斂
}

export async function getPublicSettings(): Promise<PublicSettings> {
    try {
        // 60s cache：admin 改主題後由 updateSiteTheme 的 revalidatePath 立即失效
        return await fetchApi<PublicSettings>(`${process.env.API_URL}/settings/public`, {
            next: { revalidate: 60 },
        });
    } catch {
        // 後端不可用時不擋頁面渲染，fallback 預設主題
        return {};
    }
}
