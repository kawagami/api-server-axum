"use server";

import { fetchApi } from "@/libs/fetchApi";

// 公開設定白名單（後端硬編碼 PUBLIC_KEYS）。
//
// 值**已由後端轉成該有的型別**（物件 / 陣列 / 布林 / 數字），不再是「JSON 字串包在 JSON 裡」。
// 型別仍寫 unknown 而非精確型別：值來自 runtime 設定、可由後台任意改，
// 各 resolver 本來就負責收斂壞值，宣告成精確型別只會給出假的安全感。
// （後端轉不動的值會原樣退回字串，resolver 也照樣吃得下，所以升級期間不會壞。）
export interface PublicSettings {
    site_theme?: string;          // 7 套主題之一 或 'auto'（每日輪播）
    default_color_mode?: string;  // light / dark / system
    theme_rotation?: unknown;     // 星期→主題對應表（物件），normalizeRotation 收斂
    home_features?: unknown;      // 首頁功能卡片（字串陣列），resolveHomeFeatures 收斂
    enabled_features?: unknown;   // instance 功能開關（"all" 或字串陣列），resolveEnabledFeatures 收斂
    image_client_compress?: unknown; // 前端上傳前壓縮開關（布林）
    image_client_quality?: unknown;  // 前端壓縮品質（1–100）
    image_client_max_edge?: unknown; // 前端壓縮長邊上限 px
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
