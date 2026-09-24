"use client";

import { useTranslations } from "next-intl";
import { CalendarDays, Loader2, Send, Trash2, UserPlus } from "lucide-react";
import { MAX_DAYS, ROSTER_RULES, type RosterRule } from "@/libs/roster";
import { clampNumber, digitsOnly, type Settings } from "./settings";

const RULE_LABEL_KEYS: Record<RosterRule, string> = {
    fairness: "ruleFairness",
    morning_heavy: "ruleMorning",
    night_heavy: "ruleNight",
};

const fieldClass =
    "w-full px-4 py-2 rounded-xl border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-700";
const labelClass = "text-sm font-medium text-neutral-700 dark:text-neutral-300";

/** 參數表單：名單、天數、起始日、規則，以及進階的每日人力 / 連續上班上限。狀態全在 page，這裡只顯示與回報 */
export default function RosterForm({
    settings,
    update,
    newName,
    onNewNameChange: setNewName,
    onAddNames: addNames,
    onRemoveName: removeName,
    onEnableManualSlots: enableManualSlots,
    onGenerate: handleGenerate,
    loading,
    error,
}: {
    settings: Settings;
    update: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
    newName: string;
    onNewNameChange: (value: string) => void;
    onAddNames: () => void;
    onRemoveName: (index: number) => void;
    onEnableManualSlots: () => void;
    onGenerate: () => void;
    loading: boolean;
    error: string | null;
}) {
    const t = useTranslations("Roster");
    const { names, days, rule, startDate, manualSlots, morningSlots, nightSlots, maxConsecutive } = settings;

    return (
        <section className="bg-white/60 dark:bg-neutral-800/60 backdrop-blur-md p-6 rounded-3xl shadow-lg border border-white/20">
            <h2 className="text-xl font-bold mb-4 flex items-center gap-2">
                <CalendarDays className="text-primary-500" /> {t("paramsHeading")}
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-3">
                    <label htmlFor="roster-name" className={labelClass}>
                        {t("membersLabel", { count: names.length })}
                    </label>
                    <div className="flex gap-2">
                        <input
                            id="roster-name"
                            value={newName}
                            onChange={e => setNewName(e.target.value)}
                            onKeyDown={e => {
                                if (e.key === "Enter") {
                                    e.preventDefault();
                                    addNames();
                                }
                            }}
                            placeholder={t("namePlaceholder")}
                            className={`flex-1 ${fieldClass}`}
                        />
                        <button
                            onClick={addNames}
                            aria-label={t("addName")}
                            className="p-2 bg-primary-500 text-white rounded-xl hover:bg-primary-600 transition-colors"
                        >
                            <UserPlus size={20} />
                        </button>
                    </div>
                    <p className="text-xs text-neutral-500">{t("namesHint")}</p>
                    <div className="flex flex-wrap gap-2 max-h-32 overflow-y-auto p-1">
                        {names.map((name, i) => (
                            <span
                                key={name}
                                className="px-3 py-1 bg-white/80 dark:bg-neutral-600 rounded-full text-sm flex items-center gap-2 shadow-xs border border-neutral-100 dark:border-neutral-500"
                            >
                                {name}
                                <button
                                    onClick={() => removeName(i)}
                                    aria-label={`${t("removeName")} ${name}`}
                                    className="hover:text-red-500 transition-colors"
                                >
                                    <Trash2 size={14} />
                                </button>
                            </span>
                        ))}
                    </div>
                    {names.length > 0 && (
                        <button
                            onClick={() => update("names", [])}
                            className="text-xs text-neutral-500 hover:text-red-500 transition-colors"
                        >
                            {t("clearNames")}
                        </button>
                    )}
                </div>
                <div className="space-y-4">
                    <div>
                        <label htmlFor="roster-days" className={labelClass}>{t("daysLabel")}</label>
                        <input
                            id="roster-days"
                            type="text"
                            inputMode="numeric"
                            value={days}
                            onChange={e => update("days", digitsOnly(e.target.value))}
                            onBlur={() => update("days", String(clampNumber(days, 1, MAX_DAYS, MAX_DAYS)))}
                            className={`mt-1 ${fieldClass}`}
                        />
                    </div>
                    <div>
                        <label htmlFor="roster-start" className={labelClass}>{t("startDateLabel")}</label>
                        <input
                            id="roster-start"
                            type="date"
                            value={startDate}
                            onChange={e => update("startDate", e.target.value)}
                            className={`mt-1 ${fieldClass}`}
                        />
                        <p className="mt-1 text-xs text-neutral-500">{t("startDateHint")}</p>
                    </div>
                    <div>
                        <label htmlFor="roster-rule" className={labelClass}>{t("ruleLabel")}</label>
                        <select
                            id="roster-rule"
                            value={rule}
                            onChange={e => update("rule", e.target.value as RosterRule)}
                            className={`mt-1 ${fieldClass}`}
                        >
                            {ROSTER_RULES.map(value => (
                                <option key={value} value={value}>{t(RULE_LABEL_KEYS[value])}</option>
                            ))}
                        </select>
                    </div>
                </div>
            </div>

            <details className="mt-6 rounded-2xl border border-neutral-200 dark:border-neutral-700 p-4">
                <summary className="cursor-pointer font-medium">{t("advancedHeading")}</summary>
                <div className="mt-4 space-y-4">
                    <div className="flex flex-wrap gap-2">
                        {[
                            { manual: false, label: t("slotsAuto") },
                            { manual: true, label: t("slotsManual") },
                        ].map(option => (
                            <button
                                key={option.label}
                                onClick={() => (option.manual ? enableManualSlots() : update("manualSlots", false))}
                                aria-pressed={manualSlots === option.manual}
                                className={`px-3 py-1.5 rounded-full text-sm border transition-colors ${
                                    manualSlots === option.manual
                                        ? "bg-primary-500 text-white border-primary-500"
                                        : "border-neutral-300 dark:border-neutral-600 hover:bg-white/60 dark:hover:bg-neutral-700"
                                }`}
                            >
                                {option.label}
                            </button>
                        ))}
                    </div>
                    <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                        {manualSlots && (
                            <>
                                <div>
                                    <label htmlFor="roster-morning" className={labelClass}>{t("morningSlotsLabel")}</label>
                                    <input
                                        id="roster-morning"
                                        type="text"
                                        inputMode="numeric"
                                        value={morningSlots}
                                        onChange={e => update("morningSlots", digitsOnly(e.target.value))}
                                        className={`mt-1 ${fieldClass}`}
                                    />
                                </div>
                                <div>
                                    <label htmlFor="roster-night" className={labelClass}>{t("nightSlotsLabel")}</label>
                                    <input
                                        id="roster-night"
                                        type="text"
                                        inputMode="numeric"
                                        value={nightSlots}
                                        onChange={e => update("nightSlots", digitsOnly(e.target.value))}
                                        className={`mt-1 ${fieldClass}`}
                                    />
                                </div>
                            </>
                        )}
                        <div>
                            <label htmlFor="roster-streak" className={labelClass}>{t("maxConsecutiveLabel")}</label>
                            <input
                                id="roster-streak"
                                type="text"
                                inputMode="numeric"
                                value={maxConsecutive}
                                onChange={e => update("maxConsecutive", digitsOnly(e.target.value))}
                                className={`mt-1 ${fieldClass}`}
                            />
                        </div>
                    </div>
                </div>
            </details>

            <button
                onClick={handleGenerate}
                disabled={loading || names.length === 0}
                className="w-full mt-6 py-3 bg-linear-to-r from-primary-600 to-primary-700 text-white rounded-2xl font-bold shadow-lg flex items-center justify-center gap-2 hover:opacity-90 disabled:opacity-50 transition-opacity active:scale-[0.98]"
            >
                {loading ? <Loader2 className="animate-spin" /> : <Send size={18} />}
                {loading ? t("generating") : t("generate")}
            </button>

            {error && (
                <p
                    role="alert"
                    className="mt-4 rounded-xl border border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-700 dark:text-red-400"
                >
                    {error}
                </p>
            )}
        </section>
    );
}
