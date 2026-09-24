"use client";

import { useTranslations } from "next-intl";
import { AlertTriangle, ClipboardCopy, Download, RotateCcw } from "lucide-react";
import type { RosterEntry, RosterPlan, RosterWarning } from "@/libs/roster";
import RosterDayView from "./roster-day-view";
import RosterTable from "./roster-table";

/** 後端 `RosterWarning` 機器碼 → 本頁 i18n key（後端刻意不回文案，見 `libs/roster.ts`） */
const WARNING_KEYS: Record<RosterWarning, string> = {
    understaffed: "warnUnderstaffed",
    shift_uncovered: "warnShiftUncovered",
    night_to_morning: "warnNightToMorning",
    max_consecutive_exceeded: "warnMaxConsecutiveExceeded",
};

/** 排班結果：檢視切換、複製 / 匯出、採用的每日人力與警告、手動改過的還原，以及表格本體 */
export default function RosterResult({
    entries,
    plan,
    warnings,
    startDate,
    view,
    onViewChange: setView,
    edited,
    onResetEdits,
    onCopy: copyTable,
    onExport: exportCsv,
    onToggleShift: toggleShift,
}: {
    entries: RosterEntry[];
    plan: RosterPlan;
    warnings: RosterWarning[];
    startDate: string;
    view: "person" | "day";
    onViewChange: (view: "person" | "day") => void;
    edited: boolean;
    onResetEdits: () => void;
    onCopy: () => void;
    onExport: () => void;
    onToggleShift: (personIndex: number, dayIndex: number) => void;
}) {
    const t = useTranslations("Roster");

    return (
        <section className="bg-white/70 dark:bg-neutral-900/70 backdrop-blur-lg rounded-3xl shadow-xl border border-white/30 overflow-hidden">
            <div className="p-4 sm:p-6 flex flex-col gap-3 border-b border-neutral-200 dark:border-neutral-700">
                <div className="flex flex-wrap items-center justify-between gap-3">
                    <h2 className="text-xl font-bold">{t("resultHeading")}</h2>
                    <div className="flex flex-wrap items-center gap-2 text-sm">
                        {[
                            { key: "person" as const, label: t("viewByPerson") },
                            { key: "day" as const, label: t("viewByDay") },
                        ].map(option => (
                            <button
                                key={option.key}
                                onClick={() => setView(option.key)}
                                aria-pressed={view === option.key}
                                className={`px-3 py-1.5 rounded-full border transition-colors ${
                                    view === option.key
                                        ? "bg-primary-500 text-white border-primary-500"
                                        : "border-neutral-300 dark:border-neutral-600 hover:bg-white/60 dark:hover:bg-neutral-800"
                                }`}
                            >
                                {option.label}
                            </button>
                        ))}
                        <button
                            onClick={copyTable}
                            className="px-3 py-1.5 rounded-full border border-neutral-300 dark:border-neutral-600 flex items-center gap-1.5 hover:bg-white/60 dark:hover:bg-neutral-800 transition-colors"
                        >
                            <ClipboardCopy size={14} /> {t("copyTable")}
                        </button>
                        <button
                            onClick={exportCsv}
                            className="px-3 py-1.5 rounded-full border border-neutral-300 dark:border-neutral-600 flex items-center gap-1.5 hover:bg-white/60 dark:hover:bg-neutral-800 transition-colors"
                        >
                            <Download size={14} /> {t("exportCsv")}
                        </button>
                    </div>
                </div>

                <p className="text-sm text-neutral-600 dark:text-neutral-300">
                    {t("planSummary", {
                        morning: plan.morning_slots,
                        night: plan.night_slots,
                        rest: plan.rest_slots,
                        streak: plan.max_consecutive,
                    })}
                </p>

                {warnings.length > 0 && (
                    <div
                        role="alert"
                        // 橘＝警示語意（ARCHITECTURE.md「語意色保留」列明的例外）：班表排得出來但有前提，不是錯誤
                        className="rounded-xl border border-orange-200 dark:border-orange-800 bg-orange-50 dark:bg-orange-900/20 p-3 text-sm text-orange-700 dark:text-orange-300"
                    >
                        <p className="font-semibold flex items-center gap-1.5">
                            <AlertTriangle size={15} /> {t("warningsHeading")}
                        </p>
                        <ul className="mt-1 list-disc list-inside space-y-0.5">
                            {warnings.map(code => (
                                <li key={code}>{t(WARNING_KEYS[code])}</li>
                            ))}
                        </ul>
                    </div>
                )}

                {view === "person" && (
                    <div className="flex flex-wrap items-center gap-3 text-xs text-neutral-500">
                        <span>{t("editHint")}</span>
                        {edited && (
                            <>
                                <span className="font-semibold text-orange-600 dark:text-orange-400">{t("edited")}</span>
                                <button
                                    onClick={onResetEdits}
                                    className="flex items-center gap-1 hover:text-primary-600 dark:hover:text-primary-400 transition-colors"
                                >
                                    <RotateCcw size={13} /> {t("resetEdits")}
                                </button>
                            </>
                        )}
                    </div>
                )}
            </div>

            {view === "person" ? (
                <RosterTable entries={entries} startDate={startDate} onToggle={toggleShift} />
            ) : (
                <RosterDayView entries={entries} startDate={startDate} />
            )}
        </section>
    );
}
