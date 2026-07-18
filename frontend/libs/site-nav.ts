import {
    KeyRound,
    CaseSensitive,
    TimerReset,
    CalendarDays,
    AlarmClock,
    BellRing,
    Swords,
    Crown,
    Grid3x3,
    CircleDot,
    EyeOff,
    Castle,
    Wheat,
    Crosshair,
    LayoutDashboard,
    User,
    Bell,
    TrendingUp,
    Wallet,
    ReceiptText,
    Ticket,
    type LucideIcon,
} from "lucide-react";

// 工具 / 遊戲 / 會員功能清單的單一來源：header 下拉、/tools 與 /games index 頁、dashboard 共用。
// 新增項目只要加一行（labelKey 對應 Header namespace、key 對應 ToolsHub/GamesHub/Dashboard 的 items）
// feature = 所屬的 instance 功能開關（libs/enabled-features.ts）；省略 = 核心項目不受控
export interface SiteNavItem {
    key: string;
    href: string;
    labelKey: string;
    icon: LucideIcon;
    feature?: string;
}

export const TOOLS: readonly SiteNavItem[] = [
    { key: "newPassword", href: "/tools/new-password", labelKey: "toolNewPassword", icon: KeyRound, feature: "tools" },
    { key: "convertText", href: "/tools/convert-text", labelKey: "toolConvertText", icon: CaseSensitive, feature: "tools" },
    { key: "countdown", href: "/tools/countdown", labelKey: "toolCountdown", icon: TimerReset, feature: "tools" },
    { key: "roster", href: "/tools/roster", labelKey: "toolRoster", icon: CalendarDays, feature: "roster" },
    { key: "alarm", href: "/tools/alarm", labelKey: "toolAlarm", icon: AlarmClock, feature: "tools" },
    { key: "hourlyChime", href: "/tools/hourly-chime", labelKey: "toolHourlyChime", icon: BellRing, feature: "tools" },
] as const;

export const GAMES: readonly SiteNavItem[] = [
    { key: "chess", href: "/games/chess", labelKey: "gameChess", icon: Swords, feature: "games" },
    { key: "westernChess", href: "/games/western-chess", labelKey: "gameWesternChess", icon: Crown, feature: "games" },
    { key: "gomoku", href: "/games/gomoku", labelKey: "gameGomoku", icon: Grid3x3, feature: "games" },
    { key: "go", href: "/games/go", labelKey: "gameGo", icon: CircleDot, feature: "games" },
    { key: "banqi", href: "/games/banqi", labelKey: "gameBanqi", icon: EyeOff, feature: "games" },
    { key: "avalon", href: "/games/avalon", labelKey: "gameAvalon", icon: Castle, feature: "games" },
    { key: "farm", href: "/games/farm", labelKey: "gameFarm", icon: Wheat, feature: "games" },
    { key: "metalSlug", href: "/games/metal-slug", labelKey: "gameMetalSlug", icon: Crosshair, feature: "games" },
] as const;

export const MEMBER_LINKS: readonly SiteNavItem[] = [
    { key: "dashboard", href: "/dashboard", labelKey: "dashboard", icon: LayoutDashboard },
    { key: "profile", href: "/profile", labelKey: "profile", icon: User },
    { key: "notifications", href: "/dashboard/notifications", labelKey: "notifications", icon: Bell },
    { key: "portfolio", href: "/portfolio", labelKey: "portfolio", icon: TrendingUp, feature: "portfolio" },
    { key: "ledger", href: "/ledger", labelKey: "ledger", icon: Wallet, feature: "ledger" },
    { key: "invoices", href: "/invoices", labelKey: "invoices", icon: ReceiptText, feature: "invoices" },
    { key: "lotto", href: "/lotto", labelKey: "lotto", icon: Ticket, feature: "lotto" },
] as const;

/** 依 instance 功能開關過濾導航項目（enabled 來自 resolveEnabledFeatures，null = 全開） */
export function filterNavByFeatures(
    items: readonly SiteNavItem[],
    enabled: string[] | null,
): SiteNavItem[] {
    if (enabled === null) return [...items];
    return items.filter((item) => !item.feature || enabled.includes(item.feature));
}
