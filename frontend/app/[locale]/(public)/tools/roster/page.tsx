"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslations } from "next-intl";
import {
    AlertTriangle,
    CalendarDays,
    ClipboardCopy,
    Download,
    Loader2,
    RotateCcw,
    Send,
    Trash2,
    UserPlus,
} from "lucide-react";
import { postRoster } from "@/api/tools";
import { apiErrorStatus } from "@/libs/api-error";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";
import {
    MAX_DAYS,
    MAX_NAMES,
    MAX_NAME_LEN,
    nextShift,
    parseNames,
    ROSTER_RULES,
    rosterStats,
    rosterToCsv,
    rosterToRows,
    type RosterEntry,
    type RosterPlan,
    type RosterRule,
    type RosterWarning,
} from "@/libs/roster";
import RosterDayView from "./roster-day-view";
import RosterStatsPanel from "./roster-stats";
import RosterTable from "./roster-table";

const RULE_LABEL_KEYS: Record<RosterRule, string> = {
    fairness: "ruleFairness",
    morning_heavy: "ruleMorning",
    night_heavy: "ruleNight",
};

/** 後端 `RosterWarning` 機器碼 → 本頁 i18n key（後端刻意不回文案，見 `libs/roster.ts`） */
const WARNING_KEYS: Record<RosterWarning, string> = {
    understaffed: "warnUnderstaffed",
    shift_uncovered: "warnShiftUncovered",
    night_to_morning: "warnNightToMorning",
    max_consecutive_exceeded: "warnMaxConsecutiveExceeded",
};

/**
 * 名單與參數存 localStorage：重新整理就要重打 20 個名字是這頁最大的日常痛點。
 * 版號在 key 裡，欄位改形狀時直接換號、不必寫遷移。
 */
const STORAGE_KEY = "roster_settings_v1";

/** 會被持久化的表單狀態。整包一個 state：分成八個 useState 的話 mount 後要塞八次 */
interface Settings {
    names: string[];
    days: string;
    rule: RosterRule;
    startDate: string;
    manualSlots: boolean;
    morningSlots: string;
    nightSlots: string;
    maxConsecutive: string;
}

const DEFAULT_SETTINGS: Settings = {
    names: [],
    days: String(MAX_DAYS),
    rule: "fairness",
    startDate: "",
    manualSlots: false,
    morningSlots: "",
    nightSlots: "",
    maxConsecutive: "",
};

/** 逐欄驗型：手改過或舊版格式的 localStorage 不該讓整頁掛掉，對不上就退回預設 */
function readSettings(): Settings | null {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return null;
        const saved = JSON.parse(raw) as Record<string, unknown>;
        const text = (value: unknown, fallback: string) => (typeof value === "string" ? value : fallback);
        return {
            names: Array.isArray(saved.names)
                ? saved.names
                      // 長度用字元數，跟後端的 chars 計數對齊
                      .filter((n): n is string => typeof n === "string" && [...n].length <= MAX_NAME_LEN)
                      .slice(0, MAX_NAMES)
                : DEFAULT_SETTINGS.names,
            days: text(saved.days, DEFAULT_SETTINGS.days),
            rule: ROSTER_RULES.includes(saved.rule as RosterRule) ? (saved.rule as RosterRule) : DEFAULT_SETTINGS.rule,
            startDate: text(saved.startDate, DEFAULT_SETTINGS.startDate),
            manualSlots: typeof saved.manualSlots === "boolean" ? saved.manualSlots : DEFAULT_SETTINGS.manualSlots,
            morningSlots: text(saved.morningSlots, DEFAULT_SETTINGS.morningSlots),
            nightSlots: text(saved.nightSlots, DEFAULT_SETTINGS.nightSlots),
            maxConsecutive: text(saved.maxConsecutive, DEFAULT_SETTINGS.maxConsecutive),
        };
    } catch {
        return null;
    }
}

const fieldClass =
    "w-full px-4 py-2 rounded-xl border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-700";
const labelClass = "text-sm font-medium text-neutral-700 dark:text-neutral-300";

/** 數字欄位一律存字串：存 number 的話清空欄位那一刻會被 `|| 1` 搶成 1，改不了值 */
function clampNumber(raw: string, min: number, max: number, fallback: number): number {
    const parsed = parseInt(raw, 10);
    if (Number.isNaN(parsed)) return fallback;
    return Math.min(max, Math.max(min, parsed));
}

function digitsOnly(value: string): string {
    return value.replace(/\D/g, "").slice(0, 3);
}

export default function RosterPage() {
    const t = useTranslations("Roster");
    const { toast, showToast } = useToast();

    const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
    const { names, days, rule, startDate, manualSlots, morningSlots, nightSlots, maxConsecutive } = settings;
    const update = useCallback(<K extends keyof Settings>(key: K, value: Settings[K]) => {
        setSettings(prev => ({ ...prev, [key]: value }));
    }, []);
    const [newName, setNewName] = useState("");

    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [entries, setEntries] = useState<RosterEntry[] | null>(null);
    /** 後端排出來的原始班表，手動改過之後用來「還原自動排班」 */
    const [baseline, setBaseline] = useState<RosterEntry[] | null>(null);
    const [plan, setPlan] = useState<RosterPlan | null>(null);
    const [warnings, setWarnings] = useState<RosterWarning[]>([]);
    const [view, setView] = useState<"person" | "day">("person");
    /** 讀完 localStorage 才可以回寫，否則第一輪 render 的預設值會蓋掉存檔 */
    const loaded = useRef(false);

    useEffect(() => {
        const saved = readSettings();
        // localStorage 是 client-only，mount 後才讀得到（在 render 期讀會 hydration 不一致）
        // eslint-disable-next-line react-hooks/set-state-in-effect
        if (saved) setSettings(saved);
        loaded.current = true;
    }, []);

    useEffect(() => {
        if (!loaded.current) return;
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
        } catch {
            // 私密瀏覽或配額滿：存不進去不影響功能
        }
    }, [settings]);

    const stats = useMemo(() => (entries ? rosterStats(entries) : null), [entries]);
    const rows = useMemo(
        () => (entries ? rosterToRows(entries, startDate, t("staffColumn"), n => t("dayColumn", { n })) : null),
        [entries, startDate, t],
    );

    /** 一次可加多筆：逗號／換行分隔（貼名單是這頁的主要輸入方式） */
    const addNames = () => {
        const parsed = parseNames(newName);
        if (!parsed.length) return;
        // 長度用字元數（`[...n].length`）而非 `.length`，跟後端的 chars 計數對齊
        if (parsed.some(name => [...name].length > MAX_NAME_LEN)) {
            showToast("error", t("nameTooLong", { max: MAX_NAME_LEN }));
            return;
        }
        const fresh = parsed.filter(name => !names.includes(name));
        if (!fresh.length) {
            showToast("error", t("duplicateName"));
            return;
        }
        if (names.length + fresh.length > MAX_NAMES) {
            showToast("error", t("tooManyNames", { max: MAX_NAMES }));
            return;
        }
        update("names", [...names, ...fresh]);
        setNewName("");
        if (fresh.length > 1) showToast("success", t("addedNames", { count: fresh.length }));
    };

    const removeName = (index: number) => update("names", names.filter((_, i) => i !== index));

    /**
     * 切到「自訂每日人力」時把欄位帶入現值：空欄位送出會被自己的驗證擋下
     * （早+晚 = 0），使用者得先猜兩個數字才試得動這個模式。
     * 有排過就用後端實際採用的配置，沒排過就照 fairness 的三等分。
     */
    const enableManualSlots = () => {
        const third = Math.max(1, Math.floor(names.length / 3));
        setSettings(prev => ({
            ...prev,
            manualSlots: true,
            morningSlots: prev.morningSlots || String(plan?.morning_slots ?? third),
            nightSlots: prev.nightSlots || String(plan?.night_slots ?? third),
        }));
    };

    const handleGenerate = async () => {
        if (names.length === 0) {
            showToast("error", t("emptyNames"));
            return;
        }
        const dayCount = clampNumber(days, 1, MAX_DAYS, MAX_DAYS);
        update("days", String(dayCount));

        // 前端先擋掉後端會 422 的組合：後端訊息是寫死繁中、不能直接印給使用者，
        // 只靠 422 的話這裡只能顯示一句通用錯誤，使用者不知道要改哪個欄位
        let slots: { morning_slots: number; night_slots: number } | undefined;
        if (manualSlots) {
            const morning = clampNumber(morningSlots, 0, names.length, 0);
            const night = clampNumber(nightSlots, 0, names.length, 0);
            update("morningSlots", String(morning));
            update("nightSlots", String(night));
            if (morning + night < 1) {
                showToast("error", t("slotsTooFew"));
                return;
            }
            if (morning + night > names.length) {
                showToast("error", t("slotsTooMany", { max: names.length }));
                return;
            }
            slots = { morning_slots: morning, night_slots: night };
        }
        const streakLimit = maxConsecutive ? clampNumber(maxConsecutive, 1, MAX_DAYS, MAX_DAYS) : undefined;
        if (streakLimit) update("maxConsecutive", String(streakLimit));

        setError(null);
        setLoading(true);
        try {
            const response = await postRoster({
                names,
                days: dayCount,
                rule,
                ...slots,
                ...(streakLimit ? { max_consecutive: streakLimit } : {}),
            });
            setEntries(response.data);
            setBaseline(response.data);
            setPlan(response.plan);
            setWarnings(response.warnings);
        } catch (e) {
            // 依 status 分流到自己 namespace 的 key，不要印後端訊息（那是寫死的繁中）
            const status = apiErrorStatus(e);
            setError(status === 429 ? t("errorTooMany") : status === 422 ? t("errorInvalid") : t("failed"));
        } finally {
            setLoading(false);
        }
    };

    /** 手動換班：點格子切 早班 → 晚班 → 休。統計與表尾人力都是衍生值，會即時跟著重算 */
    const toggleShift = useCallback((personIndex: number, dayIndex: number) => {
        setEntries(prev =>
            prev?.map((entry, i) =>
                i === personIndex
                    ? { ...entry, shifts: entry.shifts.map((shift, d) => (d === dayIndex ? nextShift(shift) : shift)) }
                    : entry,
            ) ?? prev,
        );
    }, []);

    const edited = Boolean(entries && baseline && JSON.stringify(entries) !== JSON.stringify(baseline));

    const exportCsv = () => {
        if (!rows) return;
        const url = URL.createObjectURL(new Blob([rosterToCsv(rows)], { type: "text/csv;charset=utf-8" }));
        const link = document.createElement("a");
        link.href = url;
        link.download = `roster-${startDate || "d1"}-${rows[0].length - 1}d.csv`;
        link.click();
        URL.revokeObjectURL(url);
    };

    const copyTable = async () => {
        if (!rows) return;
        try {
            // TSV：貼進 Excel / Google Sheets 會自動分欄
            await navigator.clipboard.writeText(rows.map(row => row.join("\t")).join("\n"));
            showToast("success", t("copied"));
        } catch {
            showToast("error", t("copyFailed"));
        }
    };

    return (
        <PageShell width="wide" className="flex flex-col gap-6">
            <PageTitle title={t("title")} />

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

            {entries && plan && (
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
                                // 橘＝警示語意（CLAUDE.md 列明的例外）：班表排得出來但有前提，不是錯誤
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
                                            onClick={() => setEntries(baseline)}
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
            )}

            {stats && <RosterStatsPanel stats={stats} />}

            <Toast toast={toast} />
        </PageShell>
    );
}
