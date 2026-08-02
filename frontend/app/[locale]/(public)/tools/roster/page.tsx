"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { CalendarDays, Send, Loader2, UserPlus, Trash2 } from "lucide-react";
import ShiftBadge from "./shift-badge";
import { postRoster } from "@/api/tools";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import Toast, { useToast } from "@/components/toast";

interface RosterEntry {
    id: string | number;
    name: string;
    shifts: string[];
}

const RULES = ["fairness", "morning_heavy", "night_heavy"] as const;
const RULE_LABEL_KEYS: Record<(typeof RULES)[number], string> = {
    fairness: "ruleFairness",
    morning_heavy: "ruleMorning",
    night_heavy: "ruleNight",
};

const fieldClass =
    "w-full px-4 py-2 rounded-xl border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-700";

export default function RosterPage() {
    const t = useTranslations("Roster");
    const [names, setNames] = useState<string[]>([]);
    const [newName, setNewName] = useState("");
    const [days, setDays] = useState(31);
    const [rule, setRule] = useState<(typeof RULES)[number]>("fairness");
    const [loading, setLoading] = useState(false);
    const [rosterData, setRosterData] = useState<RosterEntry[] | null>(null);
    const { toast, showToast } = useToast();

    const addName = () => {
        const name = newName.trim();
        if (!name) return;
        if (names.includes(name)) {
            showToast("error", t("duplicateName"));
            return;
        }
        setNames([...names, name]);
        setNewName("");
    };

    const removeName = (index: number) => setNames(names.filter((_, i) => i !== index));

    const handleGenerate = async () => {
        if (names.length === 0) {
            showToast("error", t("emptyNames"));
            return;
        }
        setLoading(true);
        try {
            const response = await postRoster({ names, days, rule });
            if (!response?.data) throw new Error("unexpected response shape");
            setRosterData(response.data as RosterEntry[]);
        } catch {
            showToast("error", t("failed"));
        } finally {
            setLoading(false);
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
                        <label htmlFor="roster-name" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">
                            {t("membersLabel", { count: names.length })}
                        </label>
                        <div className="flex gap-2">
                            <input
                                id="roster-name"
                                value={newName}
                                onChange={(e) => setNewName(e.target.value)}
                                onKeyDown={(e) => e.key === 'Enter' && addName()}
                                placeholder={t("namePlaceholder")}
                                className={`flex-1 ${fieldClass}`}
                            />
                            <button
                                onClick={addName}
                                aria-label={t("addName")}
                                className="p-2 bg-primary-500 text-white rounded-xl hover:bg-primary-600 transition-colors"
                            >
                                <UserPlus size={20} />
                            </button>
                        </div>
                        <div className="flex flex-wrap gap-2 mt-2 max-h-32 overflow-y-auto p-1">
                            {names.map((name, i) => (
                                <span key={name} className="px-3 py-1 bg-white/80 dark:bg-neutral-600 rounded-full text-sm flex items-center gap-2 shadow-xs border border-neutral-100 dark:border-neutral-500">
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
                    </div>
                    <div className="space-y-4">
                        <div>
                            <label htmlFor="roster-days" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">{t("daysLabel")}</label>
                            <input
                                id="roster-days"
                                type="number"
                                min="1"
                                max="31"
                                value={days}
                                onChange={(e) => setDays(Math.min(31, Math.max(1, parseInt(e.target.value) || 1)))}
                                className={`mt-1 ${fieldClass}`}
                            />
                        </div>
                        <div>
                            <label htmlFor="roster-rule" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">{t("ruleLabel")}</label>
                            <select
                                id="roster-rule"
                                value={rule}
                                onChange={(e) => setRule(e.target.value as (typeof RULES)[number])}
                                className={`mt-1 ${fieldClass}`}
                            >
                                {RULES.map((value) => (
                                    <option key={value} value={value}>{t(RULE_LABEL_KEYS[value])}</option>
                                ))}
                            </select>
                        </div>
                    </div>
                </div>
                <button
                    onClick={handleGenerate}
                    disabled={loading || names.length === 0}
                    className="w-full mt-6 py-3 bg-linear-to-r from-primary-600 to-primary-700 text-white rounded-2xl font-bold shadow-lg flex items-center justify-center gap-2 hover:opacity-90 disabled:opacity-50 transition-opacity active:scale-[0.98]"
                >
                    {loading ? <Loader2 className="animate-spin" /> : <Send size={18} />}
                    {loading ? t("generating") : t("generate")}
                </button>
            </section>

            {rosterData && (
                <section className="bg-white/70 dark:bg-neutral-900/70 backdrop-blur-lg rounded-3xl shadow-xl border border-white/30 overflow-hidden">
                    <div className="overflow-x-auto">
                        <table className="w-full text-left border-collapse">
                            <thead>
                                <tr className="bg-neutral-100/50 dark:bg-neutral-800/50">
                                    <th className="p-4 border-b dark:border-neutral-700 font-semibold sticky left-0 bg-white/95 dark:bg-neutral-800/95 z-20">{t("staffColumn")}</th>
                                    {rosterData[0]?.shifts.map((_, i) => (
                                        <th key={i} className="p-4 border-b dark:border-neutral-700 font-semibold text-center min-w-[90px]">
                                            <div className="text-xs text-neutral-500 font-mono">{t("dayColumn", { n: i + 1 })}</div>
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {rosterData.map((staff) => (
                                    <tr key={staff.id} className="hover:bg-white/40 dark:hover:bg-neutral-800/40 transition-colors">
                                        <td className="p-4 border-b dark:border-neutral-700 font-bold sticky left-0 bg-white/95 dark:bg-neutral-800/95 z-10">{staff.name}</td>
                                        {staff.shifts.map((shift, idx) => (
                                            <td key={idx} className="p-2 border-b dark:border-neutral-700 text-center"><ShiftBadge type={shift} /></td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                </section>
            )}

            <Toast toast={toast} />
        </PageShell>
    );
}
