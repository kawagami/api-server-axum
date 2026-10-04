"use client";

import { useMemo, useState, useTransition } from "react";
import { useLocale, useTranslations } from "next-intl";
import { ChevronDown, Loader2 } from "lucide-react";
import Toast, { useToast } from "@/components/toast";
import {
    deleteFoodLogEntry,
    getFoodLogDays,
    postFoodLogEntry,
    putFoodLogDayNote,
    putFoodLogEntry,
    type FoodLogOverview,
} from "@/api/food-log";
import type { ActionResult } from "@/libs/api-error";
import type { FoodLogEntry, FoodLogEntryInput, FoodLogSuggestion, Meal } from "@/types";
import QuickAdd from "./quick-add";
import DayCard from "./day-card";
import EntryForm, { type EntryFormInitial } from "./entry-form";
import { addDays, defaultMeal, formatAmount, minDate } from "./model";

interface Props {
    initial: FoodLogOverview;
    initialFrom: string;
    today: string;
    pageDays: number;
}

type Editor = { mode: "new"; initial: EntryFormInitial } | { mode: "edit"; entry: FoodLogEntry };

const CARD = "bg-white dark:bg-neutral-800 rounded-xl shadow-sm border dark:border-neutral-700";

export default function FoodLogClient({ initial, initialFrom, today, pageDays }: Props) {
    const t = useTranslations("FoodLog");
    const locale = useLocale();
    const { toast, showToast } = useToast();
    const [isPending, startTransition] = useTransition();

    const [overview, setOverview] = useState(initial);
    const [from, setFrom] = useState(initialFrom);
    const [date, setDate] = useState(today);
    const [meal, setMeal] = useState<Meal>(() => defaultMeal());
    const [editor, setEditor] = useState<Editor | null>(null);

    const { stores, itemsByStore } = useMemo(() => indexSuggestions(overview.suggestions), [overview.suggestions]);

    /** 寫入後重抓的範圍：時間軸至少要涵蓋剛寫入的那天（補記更早的日子時會往前延伸） */
    function rangeFor(day?: string) {
        return { from: day ? minDate(from, day) : from, to: today };
    }

    /** 套用寫入結果；失敗只出 toast。回傳是否成功（表單據此決定要不要關） */
    function apply(result: ActionResult<FoodLogOverview>, rangeFrom: string, success: string): boolean {
        if (!result.ok) {
            showToast("error", result.status === 422 ? t("errorInvalid") : t("errorSave"));
            return false;
        }
        setOverview(result.data);
        setFrom(rangeFrom);
        showToast("success", success);
        return true;
    }

    function quickAdd(s: FoodLogSuggestion) {
        startTransition(async () => {
            const range = rangeFor(date);
            const result = await postFoodLogEntry(
                { eaten_on: date, meal, store: s.store, item: s.item, quantity: s.last_quantity, amount: s.last_amount },
                range,
            );
            apply(result, range.from, t("added", { item: s.item }));
        });
    }

    async function submitEntry(input: FoodLogEntryInput): Promise<boolean> {
        const range = rangeFor(input.eaten_on);
        const result = editor?.mode === "edit"
            ? await putFoodLogEntry(editor.entry.id, input, range)
            : await postFoodLogEntry(input, range);
        const ok = apply(result, range.from, editor?.mode === "edit" ? t("saved") : t("added", { item: input.item }));
        if (ok) setEditor(null);
        return ok;
    }

    async function removeEntry(entry: FoodLogEntry): Promise<boolean> {
        if (!confirm(t("confirmDelete"))) return false;
        const range = rangeFor();
        const ok = apply(await deleteFoodLogEntry(entry.id, range), range.from, t("deleted"));
        if (ok) setEditor(null);
        return ok;
    }

    async function saveDayNote(day: string, note: string): Promise<boolean> {
        const range = rangeFor(day);
        return apply(await putFoodLogDayNote(day, note, range), range.from, t("saved"));
    }

    function loadEarlier() {
        startTransition(async () => {
            const newFrom = addDays(from, -pageDays);
            const result = await getFoodLogDays(newFrom, addDays(from, -1));
            if (!result.ok) {
                showToast("error", t("loadFailed"));
                return;
            }
            setOverview(o => ({ ...o, days: [...o.days, ...result.data] }));
            setFrom(newFrom);
        });
    }

    const { summary, days, suggestions } = overview;

    return (
        <div className="flex flex-col gap-6">
            <div className="grid grid-cols-3 gap-3">
                {(["today", "week", "month"] as const).map(key => (
                    <div key={key} className={`${CARD} px-4 py-3`}>
                        <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-1">{t(`summary.${key}`)}</p>
                        <p className="font-semibold text-lg tabular-nums">{formatAmount(summary[key], locale)}</p>
                    </div>
                ))}
            </div>

            <QuickAdd
                className={CARD}
                date={date}
                today={today}
                meal={meal}
                suggestions={suggestions}
                pending={isPending}
                onDateChange={setDate}
                onMealChange={setMeal}
                onPick={quickAdd}
                onNew={item => setEditor({ mode: "new", initial: { eaten_on: date, meal, item } })}
            />

            <section className="flex flex-col gap-3" aria-labelledby="food-log-timeline">
                <h2 id="food-log-timeline" className="text-sm font-semibold text-neutral-500 dark:text-neutral-400">
                    {t("timeline")}
                </h2>
                {days.length === 0 ? (
                    <p className="text-center text-neutral-500 dark:text-neutral-400 py-8">{t("empty")}</p>
                ) : (
                    days.map(day => (
                        <DayCard
                            key={day.date}
                            className={CARD}
                            day={day}
                            onEdit={entry => setEditor({ mode: "edit", entry })}
                            onSaveNote={note => saveDayNote(day.date, note)}
                        />
                    ))
                )}
                <button
                    type="button"
                    onClick={loadEarlier}
                    disabled={isPending}
                    className="self-center flex items-center gap-1 px-4 py-2 text-sm rounded-sm text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors"
                >
                    {isPending ? <Loader2 size={16} className="animate-spin" /> : <ChevronDown size={16} />}
                    {t("loadEarlier")}
                </button>
            </section>

            {editor && (
                <EntryForm
                    initial={editor.mode === "edit" ? editor.entry : editor.initial}
                    isEdit={editor.mode === "edit"}
                    today={today}
                    stores={stores}
                    itemsByStore={itemsByStore}
                    onSubmit={submitEntry}
                    onDelete={editor.mode === "edit" ? () => removeEntry(editor.entry) : undefined}
                    onClose={() => setEditor(null)}
                />
            )}

            <Toast toast={toast} />
        </div>
    );
}

/** 表單的店家 / 品項自動完成清單（從吃過的紀錄來，不另外維護目錄） */
function indexSuggestions(list: FoodLogSuggestion[]) {
    const itemsByStore: Record<string, string[]> = {};
    for (const s of list) {
        const key = s.store ?? "";
        (itemsByStore[key] ??= []).push(s.item);
    }
    const stores = Object.keys(itemsByStore).filter(Boolean).sort((a, b) => a.localeCompare(b));
    return { stores, itemsByStore };
}
