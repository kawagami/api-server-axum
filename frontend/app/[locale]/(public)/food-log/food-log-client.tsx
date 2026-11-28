"use client";

import { startTransition, useMemo, useOptimistic, useRef, useState, useTransition } from "react";
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
import { addDays, applyOptimistic, defaultMeal, formatAmount, isPendingEntry, minDate, type OptimisticOp } from "./model";

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
    const [isLoadingEarlier, startLoadEarlier] = useTransition();

    const [overview, setOverview] = useState(initial);
    // 快速新增 / 刪除先套到畫面，伺服器回應（整份 overview）回來後由 setOverview 取代；失敗自動退回
    const [shown, addOptimistic] = useOptimistic(overview, (o: FoodLogOverview, op: OptimisticOp) => applyOptimistic(o, op, today));
    // 時間軸已載入的最早日期；只在寫入 / 載入時讀，不參與 render
    const fromRef = useRef(initialFrom);
    const queue = useRef<Promise<unknown>>(Promise.resolve());
    const tempId = useRef(0);
    const [date, setDate] = useState(today);
    const [meal, setMeal] = useState<Meal>(() => defaultMeal());
    const [editor, setEditor] = useState<Editor | null>(null);

    const { stores, itemsByStore } = useMemo(() => indexSuggestions(overview.suggestions), [overview.suggestions]);

    /**
     * 寫入與載入一律排隊依序執行。每次寫入回的都是整份 overview，而且重抓範圍取決於上一筆
     * 寫入後的 `fromRef` —— 所以範圍要在輪到自己時才算，不能在點擊當下算。
     * （Next 本身也一次只送一個 Server Action，這裡是讓「算範圍 → 送出 → 套用」整段不交錯。）
     */
    function enqueue<T>(task: () => Promise<T>): Promise<T> {
        const run = queue.current.then(task);
        queue.current = run.catch(() => undefined);
        return run;
    }

    /** 寫入後重抓的範圍：時間軸至少要涵蓋剛寫入的那天（補記更早的日子時會往前延伸） */
    function rangeFor(day?: string) {
        return { from: day ? minDate(fromRef.current, day) : fromRef.current, to: today };
    }

    /** 排隊送出一筆寫入並套用結果；失敗只出 toast。回傳是否成功（表單據此決定要不要關） */
    function write(
        day: string | undefined,
        send: (range: { from: string; to: string }) => Promise<ActionResult<FoodLogOverview>>,
        success: string,
    ): Promise<boolean> {
        return enqueue(async () => {
            const range = rangeFor(day);
            // 網路中斷時 Server Action 本身會 throw（沒有 ActionResult 可拿）
            const result: ActionResult<FoodLogOverview> = await send(range).catch(() => ({ ok: false as const }));
            if (!result.ok) {
                showToast("error", result.status === 422 ? t("errorInvalid") : t("errorSave"));
                return false;
            }
            fromRef.current = range.from;
            // 包 transition：與樂觀狀態同一批 commit，否則暫存那筆會和伺服器回來的那筆同時出現一瞬間
            startTransition(() => setOverview(result.data));
            showToast("success", success);
            return true;
        });
    }

    function quickAdd(s: FoodLogSuggestion) {
        // 選項不再停用（樂觀更新後畫面立刻有那一筆）。同一品項還在送出中就再點，視為慢網路下的連點而忽略；
        // 真的吃兩份請改數量
        if (shown.days.some(d => d.date === date && d.entries.some(e =>
            isPendingEntry(e) && e.meal === meal && e.store === s.store && e.item === s.item))) return;

        const entry: FoodLogEntry = {
            id: -++tempId.current,
            eaten_on: date, meal, meal_label: null, store: s.store, item: s.item,
            quantity: s.last_quantity, amount: s.last_amount, rating: null, note: null,
            // 畫面不顯示時間戳；伺服器回應後整筆會被取代
            created_at: "", updated_at: "",
        };
        startTransition(async () => {
            addOptimistic({ kind: "add", entry });
            await write(
                entry.eaten_on,
                range => postFoodLogEntry(
                    { eaten_on: entry.eaten_on, meal: entry.meal, store: entry.store, item: entry.item, quantity: entry.quantity, amount: entry.amount },
                    range,
                ),
                t("added", { item: entry.item }),
            );
        });
    }

    async function submitEntry(input: FoodLogEntryInput): Promise<boolean> {
        const editing = editor?.mode === "edit" ? editor.entry : null;
        const ok = await write(
            input.eaten_on,
            range => editing ? putFoodLogEntry(editing.id, input, range) : postFoodLogEntry(input, range),
            editing ? t("saved") : t("added", { item: input.item }),
        );
        if (ok) setEditor(null);
        return ok;
    }

    /** 確認後立刻關表單、從時間軸拿掉；失敗時那筆會自動回來並出 toast */
    async function removeEntry(entry: FoodLogEntry): Promise<boolean> {
        if (!confirm(t("confirmDelete"))) return false;
        setEditor(null);
        startTransition(async () => {
            addOptimistic({ kind: "delete", entry });
            await write(undefined, range => deleteFoodLogEntry(entry.id, range), t("deleted"));
        });
        return true;
    }

    function saveDayNote(day: string, note: string): Promise<boolean> {
        return write(day, range => putFoodLogDayNote(day, note, range), t("saved"));
    }

    function loadEarlier() {
        startLoadEarlier(async () => {
            await enqueue(async () => {
                const current = fromRef.current;
                const newFrom = addDays(current, -pageDays);
                const result = await getFoodLogDays(newFrom, addDays(current, -1)).catch(() => ({ ok: false as const }));
                if (!result.ok) {
                    showToast("error", t("loadFailed"));
                    return;
                }
                fromRef.current = newFrom;
                startTransition(() => setOverview(o => ({ ...o, days: [...o.days, ...result.data] })));
            });
        });
    }

    const { summary, days, suggestions } = shown;

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
                    disabled={isLoadingEarlier}
                    className="self-center flex items-center gap-1 px-4 py-2 text-sm rounded-sm text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors"
                >
                    {isLoadingEarlier ? <Loader2 size={16} className="animate-spin" /> : <ChevronDown size={16} />}
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
