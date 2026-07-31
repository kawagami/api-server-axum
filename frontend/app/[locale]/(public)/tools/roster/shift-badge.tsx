"use client";

import { useTranslations } from "next-intl";

interface ShiftStyle {
    container: string;
    dot: string;
}

/**
 * key 是後端 `POST /roster` 回傳的班別字串（backend/src/routes/roster.rs 寫死中文），
 * 屬 API 契約，**不可改成英文代碼**；要 i18n 的是渲染出來的文字（見 SHIFT_KEY）。
 * 橘色是「未知班別」的警示語意色，屬 CLAUDE.md 列明的語意色例外。
 */
const shiftStyles: Record<string, ShiftStyle> = {
    "早班": { container: "bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-300 border-primary-200 dark:border-primary-800", dot: "bg-primary-500" },
    "晚班": { container: "bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-300 border-primary-200 dark:border-primary-800", dot: "bg-primary-500" },
    "休": { container: "bg-neutral-100 text-neutral-400 dark:bg-neutral-800/60 dark:text-neutral-500 border-neutral-200 dark:border-neutral-700", dot: "bg-neutral-400" },
    "default": { container: "bg-orange-100 text-orange-700 dark:bg-orange-900/40 dark:text-orange-300 border-orange-200 dark:border-orange-800", dot: "bg-orange-500" },
};

// 後端班別字串 → i18n key。查不到就原樣顯示後端字串（同 badge-styles 的 fallback 慣例）
const SHIFT_KEY: Record<string, "shiftMorning" | "shiftNight" | "shiftOff"> = {
    "早班": "shiftMorning",
    "晚班": "shiftNight",
    "休": "shiftOff",
};

export default function ShiftBadge({ type }: { type: string }) {
    const t = useTranslations("Roster");
    const currentStyle = shiftStyles[type] ?? shiftStyles["default"];
    const key = SHIFT_KEY[type];

    return (
        <span className={`inline-flex items-center justify-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-bold border transition-all duration-200 shadow-sm ${currentStyle.container}`}>
            <span className={`w-1.5 h-1.5 rounded-full ${currentStyle.dot}`} />
            {key ? t(key) : type}
        </span>
    );
}
