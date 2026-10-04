"use client";

import { useState } from "react";
import { useLocale, useTranslations } from "next-intl";
import { Plus, Search } from "lucide-react";
import { cn } from "@/libs/cn";
import { PUBLIC_INPUT } from "@/libs/input-styles";
import type { FoodLogSuggestion, Meal } from "@/types";
import { MEALS, formatAmount, rankSuggestions, searchSuggestions } from "./model";

interface Props {
    className: string;
    date: string;
    today: string;
    meal: Meal;
    suggestions: FoodLogSuggestion[];
    /** 寫入中：選項先停用，避免連點記成兩筆 */
    pending: boolean;
    onDateChange: (date: string) => void;
    onMealChange: (meal: Meal) => void;
    onPick: (s: FoodLogSuggestion) => void;
    /** 開新增表單，帶入搜尋框的字當品項 */
    onNew: (item: string) => void;
}

/**
 * 主要輸入路徑：吃過的東西點一下就記好（帶上次的數量與金額），沒吃過的才開表單。
 * 金額不同（漲價）就記完在時間軸上點那筆改 —— 比每次都跳確認快。
 */
export default function QuickAdd({
    className, date, today, meal, suggestions, pending, onDateChange, onMealChange, onPick, onNew,
}: Props) {
    const t = useTranslations("FoodLog");
    const [query, setQuery] = useState("");
    const searching = query.trim() !== "";
    const { frequent, recent } = rankSuggestions(suggestions, meal);
    const matches = searchSuggestions(suggestions, query);

    function pick(s: FoodLogSuggestion) {
        onPick(s);
        setQuery("");
    }

    return (
        <section className={cn(className, "p-4 flex flex-col gap-4")} aria-label={t("quickAdd")}>
            <div className="flex flex-wrap items-center gap-2">
                <input
                    type="date"
                    value={date}
                    max={today}
                    onChange={e => e.target.value && onDateChange(e.target.value)}
                    aria-label={t("date")}
                    className={PUBLIC_INPUT}
                />
                <div className="flex flex-wrap gap-1" role="group" aria-label={t("meal")}>
                    {MEALS.map(m => (
                        <button
                            key={m}
                            type="button"
                            onClick={() => onMealChange(m)}
                            aria-pressed={m === meal}
                            className={cn(
                                "px-3 py-1.5 text-sm rounded-full border transition-colors",
                                m === meal
                                    ? "bg-primary-500 border-primary-500 text-white"
                                    : "border-neutral-300 dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700",
                            )}
                        >
                            {t(`meals.${m}`)}
                        </button>
                    ))}
                </div>
            </div>

            <div className="relative">
                <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400" aria-hidden />
                <input
                    type="search"
                    value={query}
                    onChange={e => setQuery(e.target.value)}
                    placeholder={t("searchPlaceholder")}
                    aria-label={t("searchPlaceholder")}
                    className={cn(PUBLIC_INPUT, "w-full pl-9")}
                />
            </div>

            {searching ? (
                <div className="flex flex-col gap-2">
                    {matches.length > 0 ? (
                        <ChipList items={matches} pending={pending} onPick={pick} />
                    ) : (
                        <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("noMatches")}</p>
                    )}
                    <AddButton label={t("addNew", { text: query.trim() })} onClick={() => onNew(query.trim())} />
                </div>
            ) : (
                <>
                    {frequent.length > 0 && (
                        <ChipGroup title={t("frequent", { meal: t(`meals.${meal}`) })}>
                            <ChipList items={frequent} pending={pending} onPick={pick} />
                        </ChipGroup>
                    )}
                    {recent.length > 0 && (
                        <ChipGroup title={t("recent")}>
                            <ChipList items={recent} pending={pending} onPick={pick} />
                        </ChipGroup>
                    )}
                    {suggestions.length === 0 && (
                        <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("noSuggestions")}</p>
                    )}
                    <AddButton label={t("addBlank")} onClick={() => onNew("")} />
                </>
            )}
        </section>
    );
}

function ChipGroup({ title, children }: { title: string; children: React.ReactNode }) {
    return (
        <div className="flex flex-col gap-2">
            <h3 className="text-xs font-semibold text-neutral-500 dark:text-neutral-400">{title}</h3>
            {children}
        </div>
    );
}

function ChipList({ items, pending, onPick }: {
    items: FoodLogSuggestion[];
    pending: boolean;
    onPick: (s: FoodLogSuggestion) => void;
}) {
    const t = useTranslations("FoodLog");
    const locale = useLocale();
    return (
        <ul className="flex flex-wrap gap-2">
            {items.map(s => (
                <li key={`${s.store ?? ""}\u0000${s.item}`}>
                    <button
                        type="button"
                        onClick={() => onPick(s)}
                        disabled={pending}
                        className="flex flex-col items-start text-left px-3 py-1.5 rounded-lg border border-neutral-200 dark:border-neutral-600 hover:border-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30 disabled:opacity-50 transition-colors"
                    >
                        {s.store && <span className="text-xs text-neutral-500 dark:text-neutral-400">{s.store}</span>}
                        <span className="text-sm">
                            {s.item}
                            {s.last_quantity > 1 && <span className="text-neutral-500"> ×{s.last_quantity}</span>}
                            <span className="ml-2 text-neutral-500 dark:text-neutral-400 tabular-nums">
                                {s.last_amount === null ? t("noAmount") : formatAmount(s.last_amount, locale)}
                            </span>
                        </span>
                    </button>
                </li>
            ))}
        </ul>
    );
}

function AddButton({ label, onClick }: { label: string; onClick: () => void }) {
    return (
        <button
            type="button"
            onClick={onClick}
            className="self-start flex items-center gap-1 px-3 py-1.5 text-sm rounded-sm text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/30 transition-colors"
        >
            <Plus size={16} />
            {label}
        </button>
    );
}
