// 網站風格主題（非 dark/light，那個存 `theme` cookie）
// 主題值存後端 settings（key: site_theme），全站訪客生效；色階定義在 app/globals.css
export const SITE_THEMES = ['forest', 'ocean', 'sky', 'sunset', 'sakura', 'grape', 'mono'] as const;
export type SiteTheme = (typeof SITE_THEMES)[number];

// site_theme 設定可存的值：7 套主題 + 'auto'（每日輪播，依 theme_rotation 表 + 當天星期決定）
export const AUTO_THEME = 'auto' as const;
export type SiteThemeSetting = SiteTheme | typeof AUTO_THEME;

export const SITE_THEME_LABELS: Record<SiteTheme, string> = {
    forest: '森林',
    ocean: '海洋',
    sky: '天空',
    sunset: '夕陽',
    sakura: '櫻花',
    grape: '葡萄',
    mono: '單色',
};

/** 星期 → 主題對應表，key 為 "0"–"6"（0 = 週日，對齊 JS Date.getDay()） */
export type ThemeRotation = Record<string, SiteTheme>;

/** 後端 theme_rotation 缺失時的 fallback（七天各一套） */
export const DEFAULT_THEME_ROTATION: ThemeRotation = {
    '0': 'forest',
    '1': 'ocean',
    '2': 'sky',
    '3': 'sunset',
    '4': 'sakura',
    '5': 'grape',
    '6': 'mono',
};

export function isSiteTheme(value: unknown): value is SiteTheme {
    return SITE_THEMES.includes(value as SiteTheme);
}

/** 把任意值收斂成合法主題，非法 fallback forest */
export function resolveSiteTheme(value: string | undefined | null): SiteTheme {
    return isSiteTheme(value) ? (value as SiteTheme) : 'forest';
}

/** 收斂成設定值（含 'auto'）。給 admin picker 顯示目前設定用 */
export function resolveSiteThemeSetting(value: string | undefined | null): SiteThemeSetting {
    if (value === AUTO_THEME) return AUTO_THEME;
    return resolveSiteTheme(value);
}

/** 把後端傳來的 theme_rotation（物件或 JSON 字串）正規化成 ThemeRotation；只保留合法主題值 */
export function normalizeRotation(raw: unknown): ThemeRotation {
    let obj: unknown = raw;
    if (typeof raw === 'string') {
        try {
            obj = JSON.parse(raw);
        } catch {
            return { ...DEFAULT_THEME_ROTATION };
        }
    }
    if (!obj || typeof obj !== 'object') return { ...DEFAULT_THEME_ROTATION };

    const result: ThemeRotation = { ...DEFAULT_THEME_ROTATION };
    for (const day of ['0', '1', '2', '3', '4', '5', '6']) {
        const v = (obj as Record<string, unknown>)[day];
        if (isSiteTheme(v)) result[day] = v as SiteTheme;
    }
    return result;
}

/** 以 Asia/Taipei 算今天星期幾（0 = 週日 .. 6 = 週六），釘時區避免吃 server UTC 半夜換錯天 */
export function getTaipeiWeekday(): number {
    const short = new Intl.DateTimeFormat('en-US', {
        timeZone: 'Asia/Taipei',
        weekday: 'short',
    }).format(new Date());
    return ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'].indexOf(short);
}

/**
 * 解析「當下實際要套用的主題」：
 * - site_theme 是具體主題 → 固定該主題
 * - site_theme 是 'auto'    → 依 theme_rotation 表 + 當天（Asia/Taipei）星期決定
 */
export function resolveActiveTheme(
    setting: string | undefined | null,
    rotation: ThemeRotation,
): SiteTheme {
    if (setting === AUTO_THEME) {
        return resolveSiteTheme(rotation[String(getTaipeiWeekday())]);
    }
    return resolveSiteTheme(setting);
}

/** Client-only：即時套用 data-theme（樂觀更新；持久化走 updateSiteTheme server action） */
export function applySiteThemeAttr(theme: SiteTheme) {
    document.documentElement.setAttribute('data-theme', theme);
}
