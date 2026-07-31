"use client";

import { useState } from "react";
import { Sun, Moon, SunMoon } from "lucide-react";
import { applyUserColorMode, clearUserColorMode, type UserColorMode } from "@/libs/color-mode";

interface Props {
    /** SSR 當下的使用者選擇（auto = 無 cookie，跟隨網站預設） */
    initialMode: UserColorMode;
    /** 網站預設是否深色；null = 跟隨系統 */
    defaultIsDark: boolean | null;
    /**
     * 三態的 aria-label／title。
     * 本元件前後台共用，而 admin 不在 [locale] 之下、沒有 NextIntlClientProvider，
     * 所以**不能**在這裡呼叫 useTranslations —— 文案由呼叫端給：
     * 前台 Header 傳翻譯後的字串，admin 省略此 prop 吃預設繁中（後台文案一律繁中）。
     */
    labels?: Record<UserColorMode, string>;
}

const NEXT_MODE: Record<UserColorMode, UserColorMode> = {
    light: 'dark',
    dark: 'auto',
    auto: 'light',
};

const DEFAULT_MODE_LABEL: Record<UserColorMode, string> = {
    light: '淺色模式（點擊切深色）',
    dark: '深色模式（點擊改跟隨網站預設）',
    auto: '跟隨網站預設（點擊切淺色）',
};

export default function ThemeButton({ initialMode, defaultIsDark, labels }: Props) {
    const [mode, setMode] = useState<UserColorMode>(initialMode);
    const MODE_LABEL = labels ?? DEFAULT_MODE_LABEL;

    function cycle() {
        const next = NEXT_MODE[mode];
        setMode(next);
        if (next === 'auto') {
            clearUserColorMode(defaultIsDark);
        } else {
            applyUserColorMode(next);
        }
    }

    return (
        <div className="flex items-center">
            <button
                className="w-8 h-8 bg-neutral-400 dark:bg-white text-white dark:text-neutral-700 rounded-full grid place-content-center hover:scale-110 transition-transform"
                onClick={cycle}
                aria-label={MODE_LABEL[mode]}
                title={MODE_LABEL[mode]}
            >
                {mode === 'light' && <Sun size={18} />}
                {mode === 'dark' && <Moon size={18} />}
                {mode === 'auto' && <SunMoon size={18} />}
            </button>
        </div>
    );
}
