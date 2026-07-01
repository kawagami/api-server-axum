// 深淺色模式（dark / light），與風格主題（site-theme）正交
// 優先序：使用者 theme cookie ＞ admin 設定 default_color_mode ＞ 系統 prefers-color-scheme
export type ColorMode = 'light' | 'dark';
export type ColorModeSetting = ColorMode | 'system';
/** 使用者目前的選擇：明確亮/暗，或 auto（無 cookie，跟隨網站預設） */
export type UserColorMode = ColorMode | 'auto';

export function resolveDefaultColorMode(value: string | undefined | null): ColorModeSetting {
    return value === 'dark' || value === 'light' ? value : 'system';
}

/** Client-only：寫 cookie 並套用 class */
export function applyUserColorMode(mode: ColorMode) {
    document.cookie = `theme=${mode}; path=/; max-age=31536000`;
    document.documentElement.classList.toggle('dark', mode === 'dark');
}

/** Client-only：清掉個人偏好，回到網站預設（defaultIsDark=null 表示跟隨系統） */
export function clearUserColorMode(defaultIsDark: boolean | null) {
    document.cookie = 'theme=; path=/; max-age=0';
    const dark = defaultIsDark ?? window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.classList.toggle('dark', dark);
}
