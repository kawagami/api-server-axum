"use client";

import { useState } from "react";
import { Sun, Moon, SunMoon } from "lucide-react";
import { applyUserColorMode, clearUserColorMode, type UserColorMode } from "@/libs/color-mode";

interface Props {
    /** SSR 當下的使用者選擇（auto = 無 cookie，跟隨網站預設） */
    initialMode: UserColorMode;
    /** 網站預設是否深色；null = 跟隨系統 */
    defaultIsDark: boolean | null;
}

const NEXT_MODE: Record<UserColorMode, UserColorMode> = {
    light: 'dark',
    dark: 'auto',
    auto: 'light',
};

const MODE_LABEL: Record<UserColorMode, string> = {
    light: '淺色模式（點擊切深色）',
    dark: '深色模式（點擊改跟隨網站預設）',
    auto: '跟隨網站預設（點擊切淺色）',
};

export default function ThemeButton({ initialMode, defaultIsDark }: Props) {
    const [mode, setMode] = useState<UserColorMode>(initialMode);

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
                className="w-8 h-8 bg-neutral-400 dark:bg-white text-white dark:text-neutral-700 rounded-full grid place-content-center focus:outline-none focus:ring-2 focus:ring-primary-400 hover:scale-110 transition-transform"
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
