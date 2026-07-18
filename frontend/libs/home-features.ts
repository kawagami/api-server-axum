import {
    FileText,
    GraduationCap,
    Gamepad2,
    Wallet,
    TrendingUp,
    ReceiptText,
    Ticket,
    Wrench,
    Info,
    type LucideIcon,
} from "lucide-react";

// 首頁功能卡片 registry：icon / href 的唯一來源。
// 顯示與排序由後端設定 home_features（JSON 字串陣列）控制，admin settings 頁管理。
// 新增卡片＝此處加一行 + messages/*.json 的 Home.features.{key}（三語系同步）。
// label 僅供後台管理顯示（前台標題走 i18n），比照 libs/site-theme.ts 的 SITE_THEME_LABELS。
// feature = 所屬的 instance 功能開關（libs/enabled-features.ts）；省略 = 不受控（如 about）。
// 首頁實際顯示 = home_features（展示與排序）∩ enabled_features（站有沒有這功能）。
export interface HomeFeature {
    key: string;
    href: string;
    icon: LucideIcon;
    label: string;
    feature?: string;
}

export const HOME_FEATURES: readonly HomeFeature[] = [
    { key: "blog", href: "/blogs", icon: FileText, label: "文章", feature: "blog" },
    { key: "vocab", href: "/vocab", icon: GraduationCap, label: "單字闖關", feature: "vocab" },
    { key: "games", href: "/games", icon: Gamepad2, label: "對戰遊戲", feature: "games" },
    { key: "ledger", href: "/ledger", icon: Wallet, label: "記帳本", feature: "ledger" },
    { key: "portfolio", href: "/portfolio", icon: TrendingUp, label: "投資組合", feature: "portfolio" },
    { key: "invoices", href: "/invoices", icon: ReceiptText, label: "發票對獎", feature: "invoices" },
    { key: "lotto", href: "/lotto", icon: Ticket, label: "樂透對獎", feature: "lotto" },
    { key: "tools", href: "/tools", icon: Wrench, label: "實用工具", feature: "tools" },
    { key: "about", href: "/about", icon: Info, label: "關於本站" },
] as const;

// 解析後端 home_features 設定值：過濾未知 key、去重。
// 缺值或 parse 失敗 → fallback 全部顯示（後台改壞不擋首頁）；合法空陣列 = 刻意全部隱藏。
export function resolveHomeFeatures(setting: unknown): HomeFeature[] {
    let keys: unknown = setting;
    if (typeof setting === "string") {
        try {
            keys = JSON.parse(setting);
        } catch {
            return [...HOME_FEATURES];
        }
    }
    if (!Array.isArray(keys)) return [...HOME_FEATURES];

    const byKey = new Map(HOME_FEATURES.map((f) => [f.key, f]));
    const seen = new Set<string>();
    const result: HomeFeature[] = [];
    for (const key of keys) {
        if (typeof key !== "string" || seen.has(key)) continue;
        const feature = byKey.get(key);
        if (feature) {
            result.push(feature);
            seen.add(key);
        }
    }
    return result;
}
