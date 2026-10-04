"use client";

import { useTranslations } from "next-intl";
import { Star } from "lucide-react";
import { cn } from "@/libs/cn";

const LEVELS = [1, 2, 3, 4, 5] as const;

/** 唯讀星等（時間軸上用） */
export function Stars({ value, className }: { value: number; className?: string }) {
    const t = useTranslations("FoodLog");
    return (
        <span className={cn("inline-flex align-[-2px] gap-px text-primary-500", className)} role="img" aria-label={t("starLabel", { n: value })}>
            {LEVELS.map(n => (
                <Star key={n} size={12} aria-hidden fill={n <= value ? "currentColor" : "none"} className={n <= value ? "" : "opacity-40"} />
            ))}
        </span>
    );
}

/** 點星評分；再點目前那顆 = 清除（評分是選填） */
export function RatingInput({ value, onChange }: { value: number | null; onChange: (v: number | null) => void }) {
    const t = useTranslations("FoodLog");
    return (
        <div className="flex items-center gap-1" role="group" aria-label={t("rating")}>
            {LEVELS.map(n => (
                <button
                    key={n}
                    type="button"
                    onClick={() => onChange(value === n ? null : n)}
                    aria-pressed={value !== null && n <= value}
                    aria-label={t("starLabel", { n })}
                    className="p-1 rounded-sm text-primary-500 hover:bg-primary-50 dark:hover:bg-primary-900/30 transition-colors"
                >
                    <Star size={22} fill={value !== null && n <= value ? "currentColor" : "none"} aria-hidden />
                </button>
            ))}
        </div>
    );
}
