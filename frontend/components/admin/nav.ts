import {
    FileText,
    Users,
    TrendingUp,
    Wrench,
    ScrollText,
    Settings,
    type LucideIcon,
} from "lucide-react";

export interface AdminNavItem {
    label: string;
    href: string;
    // 顯示此入口所需的權限（xxx:read）；省略＝所有登入管理員皆可見（如修改密碼）
    permission?: string;
    // 所屬的 instance 功能開關（libs/enabled-features.ts）；省略＝核心項目不受控
    feature?: string;
}

export interface AdminNavGroup {
    label: string;
    icon: LucideIcon;
    items: AdminNavItem[];
}

// Admin 導航單一來源：sidebar 與首頁 quick links 共用
export const adminNavGroups: AdminNavGroup[] = [
    {
        label: "內容",
        icon: FileText,
        items: [
            { label: "文章", href: "/admin/blogs", permission: "blog:read", feature: "blog" },
            { label: "圖片", href: "/admin/images", permission: "image:read", feature: "blog" },
            { label: "單字題庫", href: "/admin/vocab", permission: "vocab:read", feature: "vocab" },
        ],
    },
    {
        label: "股票",
        icon: TrendingUp,
        items: [
            { label: "列表", href: "/admin/stocks/list", permission: "stock:read", feature: "stocks" },
            { label: "回購計畫", href: "/admin/stocks/get-buyback-plans", permission: "stock:read", feature: "stocks" },
            { label: "未完成回購", href: "/admin/stocks/get-unfinished-buyback-price-gap", permission: "stock:read", feature: "stocks" },
            { label: "收盤價查詢", href: "/admin/stocks/fetch-stock-closing-price-pair", permission: "stock:read", feature: "stocks" },
            { label: "當日全部", href: "/admin/stocks/stock-day-all", permission: "stock:read", feature: "stocks" },
        ],
    },
    {
        label: "會員與權限",
        icon: Users,
        items: [
            { label: "會員列表", href: "/admin/members", permission: "member:read" },
            { label: "管理員", href: "/admin/users", permission: "user:read" },
            { label: "角色", href: "/admin/roles", permission: "role:read" },
        ],
    },
    {
        label: "工具",
        icon: Wrench,
        items: [
            { label: "WS", href: "/admin/ws", permission: "ws:read" },
            { label: "對局總覽", href: "/admin/games", permission: "game:read", feature: "games" },
            { label: "Torrents", href: "/admin/torrents", permission: "torrent:read", feature: "torrents" },
        ],
    },
    {
        label: "觀測",
        icon: ScrollText,
        items: [
            { label: "政府標案", href: "/admin/gov_tenders", permission: "gov_tender:read", feature: "gov_tenders" },
            { label: "到訪統計", href: "/admin/stats", permission: "stat:read" },
            { label: "系統指標", href: "/admin/metrics", permission: "metric:read" },
            { label: "Logs", href: "/admin/logs", permission: "log:read" },
            { label: "Audit Logs", href: "/admin/audit_logs", permission: "audit:read" },
        ],
    },
    {
        label: "設定",
        icon: Settings,
        items: [
            { label: "Settings", href: "/admin/settings", permission: "setting:read" },
            { label: "平台設定", href: "/admin/platform", permission: "platform:read" },
            { label: "修改密碼", href: "/admin/change-password" },
            { label: "Passkey 管理", href: "/admin/passkeys" },
        ],
    },
];

/**
 * 依權限＋instance 功能開關過濾導航：保留無 permission 標記或權限符合、且功能未被關閉的項目；
 * 項目全被過濾掉的分組整組移除。super_admin 的 permissions 含全部權限 → 全見。
 * enabledFeatures 來自 resolveEnabledFeatures（null = 全開）。
 * 純函式，供 sidebar（client）與首頁 quick links（server）共用。
 */
export function filterNavByPermissions(
    groups: AdminNavGroup[],
    permissions: string[],
    enabledFeatures: string[] | null = null,
): AdminNavGroup[] {
    const allowed = new Set(permissions);
    return groups
        .map(group => ({
            ...group,
            items: group.items.filter(item =>
                (!item.permission || allowed.has(item.permission)) &&
                (!item.feature || enabledFeatures === null || enabledFeatures.includes(item.feature))
            ),
        }))
        .filter(group => group.items.length > 0);
}
