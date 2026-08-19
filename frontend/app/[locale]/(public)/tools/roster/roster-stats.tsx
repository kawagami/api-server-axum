"use client";

import { useTranslations } from "next-intl";
import { AlertTriangle, CheckCircle2 } from "lucide-react";
import type { RosterStats } from "@/libs/roster";

/**
 * 每人班數與最長連續上班天數，加一行「每天每班都有人 / 有 N 天班別 0 人」。
 *
 * 這一區是**缺陷的自我檢查介面**：舊版排班的人力洞與工時偏誤都存在，但畫面上
 * 完全看不出來，使用者只能一格一格數。手動改班之後這裡即時重算（純函式，不打 API）。
 */
export default function RosterStatsPanel({ stats }: { stats: RosterStats }) {
    const t = useTranslations("Roster");
    const gaps = stats.gapDays;

    return (
        <section className="bg-white/60 dark:bg-neutral-800/60 backdrop-blur-md p-6 rounded-3xl shadow-lg border border-white/20">
            <h2 className="text-xl font-bold mb-4">{t("statsHeading")}</h2>

            <p
                className={`flex items-center gap-2 text-sm mb-4 ${gaps ? "text-orange-600 dark:text-orange-400" : "text-neutral-600 dark:text-neutral-300"}`}
            >
                {gaps ? <AlertTriangle size={16} className="shrink-0" /> : <CheckCircle2 size={16} className="shrink-0" />}
                {gaps ? t("statsGapDays", { count: gaps }) : t("statsNoGap")}
            </p>

            <div className="overflow-x-auto">
                <table className="w-full text-sm text-left border-collapse">
                    <caption className="sr-only">{t("statsHeading")}</caption>
                    <thead>
                        <tr className="text-neutral-500">
                            <th scope="col" className="py-2 pr-4 font-semibold">{t("statsPerson")}</th>
                            {[t("statsWork"), t("statsMorning"), t("statsNight"), t("statsOff"), t("statsMaxStreak")].map(label => (
                                <th key={label} scope="col" className="py-2 px-3 font-semibold text-right whitespace-nowrap">{label}</th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {stats.perPerson.map(person => (
                            <tr key={person.name} className="border-t border-neutral-200 dark:border-neutral-700">
                                <th scope="row" className="py-2 pr-4 font-medium text-left">{person.name}</th>
                                {[person.work, person.morning, person.night, person.off, person.maxStreak].map((value, i) => (
                                    <td key={i} className="py-2 px-3 text-right font-mono">{value}</td>
                                ))}
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </section>
    );
}
