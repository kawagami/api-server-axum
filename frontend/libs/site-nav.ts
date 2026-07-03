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
    type LucideIcon,
} from "lucide-react";

// 工具與遊戲清單的單一來源：header 下拉、/tools 與 /games index 頁共用。
// 新增項目只要加一行（labelKey 對應 Header namespace、key 對應 ToolsHub/GamesHub 的 items）
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
