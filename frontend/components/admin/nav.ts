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
            { label: "文章", href: "/admin/blogs" },
            { label: "圖片", href: "/admin/images" },
        ],
    },
    {
        label: "股票",
        icon: TrendingUp,
        items: [
            { label: "列表", href: "/admin/stocks/list" },
            { label: "回購計畫", href: "/admin/stocks/get-buyback-plans" },
            { label: "未完成回購", href: "/admin/stocks/get-unfinished-buyback-price-gap" },
            { label: "收盤價查詢", href: "/admin/stocks/fetch-stock-closing-price-pair" },
            { label: "當日全部", href: "/admin/stocks/stock-day-all" },
        ],
    },
    {
        label: "會員與權限",
        icon: Users,
        items: [
            { label: "會員列表", href: "/admin/members" },
            { label: "管理員", href: "/admin/users" },
            { label: "角色", href: "/admin/roles" },
        ],
    },
    {
        label: "工具",
        icon: Wrench,
        items: [
            { label: "WS", href: "/admin/ws" },
            { label: "對局總覽", href: "/admin/games" },
            { label: "Torrents", href: "/admin/torrents" },
        ],
    },
    {
        label: "觀測",
        icon: ScrollText,
        items: [
            { label: "到訪統計", href: "/admin/stats" },
            { label: "Logs", href: "/admin/logs" },
            { label: "Audit Logs", href: "/admin/audit_logs" },
        ],
    },
    {
        label: "設定",
        icon: Settings,
        items: [
            { label: "Settings", href: "/admin/settings" },
            { label: "修改密碼", href: "/admin/change-password" },
        ],
    },
];
