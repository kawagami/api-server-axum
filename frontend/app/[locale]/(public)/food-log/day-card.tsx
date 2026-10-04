"use client";

import { useState } from "react";
import { useLocale, useTranslations } from "next-intl";
import { NotebookPen } from "lucide-react";
import { cn } from "@/libs/cn";
import { PUBLIC_INPUT } from "@/libs/input-styles";
import type { FoodLogDay, FoodLogEntry } from "@/types";
import { MEALS, formatAmount } from "./model";
import { Stars } from "./rating";

interface Props {
    className: string;
    day: FoodLogDay;
    onEdit: (entry: FoodLogEntry) => void;
    onSaveNote: (note: string) => Promise<boolean>;
}

/** 時間軸的一天：依餐別分組列出品項（點一筆開編輯），底部是當天備註 */
export default function DayCard({ className, day, onEdit, onSaveNote }: Props) {
    const t = useTranslations("FoodLog");
    const locale = useLocale();
    // 正午 +08:00：日期字串本身沒有時區，取當天中間避免任何時區換算跨日
    const heading = new Intl.DateTimeFormat(locale, {
        month: "long",
        day: "numeric",
        weekday: "short",
        timeZone: "Asia/Taipei",
    }).format(new Date(`${day.date}T12:00:00+08:00`));

    return (
        <article className={cn(className, "p-4 flex flex-col gap-3")}>
            <header className="flex items-baseline justify-between gap-2">
                <h3 className="font-semibold">{heading}</h3>
                <span className="text-sm text-neutral-500 dark:text-neutral-400 tabular-nums">
                    {t("dayTotal")} <span className="font-semibold text-neutral-800 dark:text-neutral-100">{formatAmount(day.total, locale)}</span>
                </span>
            </header>

            {MEALS.map(meal => {
                const entries = day.entries.filter(e => e.meal === meal);
                if (entries.length === 0) return null;
                return (
                    <div key={meal} className="flex flex-col gap-1">
                        <h4 className="text-xs font-semibold text-primary-600 dark:text-primary-400">{t(`meals.${meal}`)}</h4>
                        <ul className="flex flex-col">
                            {entries.map(e => (
                                <li key={e.id}>
                                    <EntryRow entry={e} onClick={() => onEdit(e)} />
                                </li>
                            ))}
                        </ul>
                    </div>
                );
            })}

            <DayNote note={day.note} onSave={onSaveNote} />
        </article>
    );
}

function EntryRow({ entry, onClick }: { entry: FoodLogEntry; onClick: () => void }) {
    const t = useTranslations("FoodLog");
    const locale = useLocale();
    return (
        <button
            type="button"
            onClick={onClick}
            aria-label={t("editEntryOf", { item: entry.item })}
            className="w-full flex items-start justify-between gap-3 text-left rounded-sm px-2 py-1.5 -mx-2 hover:bg-neutral-100 dark:hover:bg-neutral-700/50 transition-colors"
        >
            <span className="flex flex-col min-w-0">
                <span className="text-sm">
                    {entry.item}
                    {entry.quantity > 1 && <span className="text-neutral-500"> ×{entry.quantity}</span>}
                    {entry.rating !== null && <Stars value={entry.rating} className="ml-2" />}
                </span>
                {(entry.store || entry.meal_label) && (
                    <span className="text-xs text-neutral-500 dark:text-neutral-400">
                        {[entry.store, entry.meal_label].filter(Boolean).join(" · ")}
                    </span>
                )}
                {entry.note && (
                    <span className="text-xs text-neutral-600 dark:text-neutral-300 whitespace-pre-line mt-0.5">{entry.note}</span>
                )}
            </span>
            <span className="text-sm tabular-nums shrink-0 text-neutral-700 dark:text-neutral-200">
                {entry.amount === null ? t("noAmount") : formatAmount(entry.amount, locale)}
            </span>
        </button>
    );
}

/** 當天備註（跑步、換牙刷…）：就地編輯，清空即刪除 */
function DayNote({ note, onSave }: { note: string | null; onSave: (note: string) => Promise<boolean> }) {
    const t = useTranslations("FoodLog");
    const [editing, setEditing] = useState(false);
    const [draft, setDraft] = useState(note ?? "");
    const [saving, setSaving] = useState(false);

    async function save() {
        setSaving(true);
        try {
            if (await onSave(draft)) setEditing(false);
        } finally {
            setSaving(false);
        }
    }

    if (!editing) {
        return (
            <button
                type="button"
                onClick={() => { setDraft(note ?? ""); setEditing(true); }}
                className="flex items-start gap-2 text-left text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-600 dark:hover:text-primary-400 border-t dark:border-neutral-700 pt-2 transition-colors"
            >
                <NotebookPen size={14} className="mt-0.5 shrink-0" aria-hidden />
                {note ? (
                    <span className="whitespace-pre-line text-neutral-700 dark:text-neutral-200">{note}</span>
                ) : (
                    <span>{t("addDayNote")}</span>
                )}
            </button>
        );
    }

    return (
        <div className="flex flex-col gap-2 border-t dark:border-neutral-700 pt-2">
            <textarea
                value={draft}
                onChange={e => setDraft(e.target.value)}
                rows={2}
                autoFocus
                placeholder={t("dayNotePlaceholder")}
                aria-label={t("dayNote")}
                className={cn(PUBLIC_INPUT, "w-full")}
            />
            <div className="flex justify-end gap-2">
                <button
                    type="button"
                    onClick={() => setEditing(false)}
                    className="px-3 py-1.5 text-sm rounded-sm border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                >
                    {t("cancel")}
                </button>
                <button
                    type="button"
                    onClick={save}
                    disabled={saving}
                    className="px-3 py-1.5 text-sm rounded-sm bg-primary-500 text-white hover:bg-primary-600 disabled:opacity-50"
                >
                    {t("save")}
                </button>
            </div>
        </div>
    );
}
