"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslations } from "next-intl";
import {
    MAX_DAYS,
    MAX_NAMES,
    MAX_NAME_LEN,
    nextShift,
    parseNames,
    rosterStats,
    rosterToCsv,
    rosterToRows,
    type RosterEntry,
    type RosterPlan,
    type RosterWarning,
} from "@/libs/roster";
import { postRoster } from "@/api/tools";
import { apiErrorStatus } from "@/libs/api-error";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";
import RosterForm from "./roster-form";
import RosterResult from "./roster-result";
import RosterStatsPanel from "./roster-stats";
import { clampNumber, DEFAULT_SETTINGS, readSettings, STORAGE_KEY, type Settings } from "./settings";

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

            <RosterForm
                settings={settings}
                update={update}
                newName={newName}
                onNewNameChange={setNewName}
                onAddNames={addNames}
                onRemoveName={removeName}
                onEnableManualSlots={enableManualSlots}
                onGenerate={handleGenerate}
                loading={loading}
                error={error}
            />

            {entries && plan && (
                <RosterResult
                    entries={entries}
                    plan={plan}
                    warnings={warnings}
                    startDate={startDate}
                    view={view}
                    onViewChange={setView}
                    edited={edited}
                    onResetEdits={() => setEntries(baseline)}
                    onCopy={copyTable}
                    onExport={exportCsv}
                    onToggleShift={toggleShift}
                />
            )}

            {stats && <RosterStatsPanel stats={stats} />}

            <Toast toast={toast} />
        </PageShell>
    );
}
