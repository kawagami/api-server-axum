// instance 級功能開關：後端 app_settings 的 enabled_features（"all" 或 JSON 字串陣列），
// GET /settings/public 下發。key 權威在後端 Feature enum（backend/src/structs/features.rs），
// 此清單只給後台 picker 列選項用，兩邊新增功能要同步。
// 與 home_features 職責不同：enabled_features = 這個站有沒有這功能（影響 API 404），
// home_features = 首頁展示哪些卡片與排序（純展示），首頁卡片取兩者交集。
export const BACKEND_FEATURES = [
    { key: "blog", label: "文章" },
    { key: "tools", label: "實用工具" },
    { key: "roster", label: "排班工具" },
    { key: "games", label: "對戰遊戲" },
    { key: "stocks", label: "股票追蹤" },
    { key: "portfolio", label: "投資組合" },
    { key: "ledger", label: "記帳本" },
    { key: "invoices", label: "發票對獎" },
    { key: "lotto", label: "樂透對獎" },
    { key: "vocab", label: "單字闖關" },
    { key: "torrents", label: "Torrent 下載" },
    { key: "gov_tenders", label: "政府標案" },
    { key: "message", label: "訪客留言" },
] as const;

export type BackendFeatureKey = (typeof BACKEND_FEATURES)[number]["key"];

/**
 * 解析後端 enabled_features 設定值。
 * `null` = 全開（值為 "all"、缺值或壞值 —— 後端 PATCH 已嚴格驗證，fail-open 不擋站）。
 * 回傳陣列而非 Set：要跨 server → client component 邊界（Header props）。
 */
export function resolveEnabledFeatures(setting: unknown): string[] | null {
    if (typeof setting !== "string" || setting === "all") return null;
    try {
        const parsed = JSON.parse(setting);
        if (!Array.isArray(parsed)) return null;
        return parsed.filter((k): k is string => typeof k === "string");
    } catch {
        return null;
    }
}

/** feature 為 undefined = 不受開關控制（核心項目），一律顯示 */
export function isFeatureEnabled(enabled: string[] | null, feature?: string): boolean {
    if (!feature || enabled === null) return true;
    return enabled.includes(feature);
}
