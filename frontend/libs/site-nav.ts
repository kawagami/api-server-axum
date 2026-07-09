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
export interface SiteNavItem {
    key: string;
    href: string;
    labelKey: string;
    icon: LucideIcon;
}

export const TOOLS: readonly SiteNavItem[] = [
    { key: "newPassword", href: "/tools/new-password", labelKey: "toolNewPassword", icon: KeyRound },
    { key: "convertText", href: "/tools/convert-text", labelKey: "toolConvertText", icon: CaseSensitive },
    { key: "countdown", href: "/tools/countdown", labelKey: "toolCountdown", icon: TimerReset },
    { key: "roster", href: "/tools/roster", labelKey: "toolRoster", icon: CalendarDays },
    { key: "alarm", href: "/tools/alarm", labelKey: "toolAlarm", icon: AlarmClock },
    { key: "hourlyChime", href: "/tools/hourly-chime", labelKey: "toolHourlyChime", icon: BellRing },
] as const;

export const GAMES: readonly SiteNavItem[] = [
    { key: "chess", href: "/games/chess", labelKey: "gameChess", icon: Swords },
    { key: "westernChess", href: "/games/western-chess", labelKey: "gameWesternChess", icon: Crown },
    { key: "gomoku", href: "/games/gomoku", labelKey: "gameGomoku", icon: Grid3x3 },
    { key: "go", href: "/games/go", labelKey: "gameGo", icon: CircleDot },
    { key: "banqi", href: "/games/banqi", labelKey: "gameBanqi", icon: EyeOff },
    { key: "avalon", href: "/games/avalon", labelKey: "gameAvalon", icon: Castle },
    { key: "farm", href: "/games/farm", labelKey: "gameFarm", icon: Wheat },
    { key: "metalSlug", href: "/games/metal-slug", labelKey: "gameMetalSlug", icon: Crosshair },
] as const;

export const MEMBER_LINKS: readonly SiteNavItem[] = [
    { key: "dashboard", href: "/dashboard", labelKey: "dashboard", icon: LayoutDashboard },
    { key: "profile", href: "/profile", labelKey: "profile", icon: User },
    { key: "notifications", href: "/dashboard/notifications", labelKey: "notifications", icon: Bell },
    { key: "portfolio", href: "/portfolio", labelKey: "portfolio", icon: TrendingUp },
    { key: "ledger", href: "/ledger", labelKey: "ledger", icon: Wallet },
    { key: "invoices", href: "/invoices", labelKey: "invoices", icon: ReceiptText },
    { key: "lotto", href: "/lotto", labelKey: "lotto", icon: Ticket },
] as const;
