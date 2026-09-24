"use client";

import type { VocabLeaderboard, VocabLeaderboardPeriod, VocabLeaderboardRow } from "@/types";
import { Loader2, Trophy } from "lucide-react";
import { useState } from "react";
import type { T } from "./ui";

const PERIODS: VocabLeaderboardPeriod[] = ["weekly", "monthly", "all"];
const PERIOD_KEYS: Record<VocabLeaderboardPeriod, "lbWeekly" | "lbMonthly" | "lbAll"> = {
    weekly: "lbWeekly", monthly: "lbMonthly", all: "lbAll",
};

/** OAuth 頭像是外部 URL,壞圖要退回首字母而不是留一個破圖示 */
function Avatar({ row }: { row: VocabLeaderboardRow }) {
    const [broken, setBroken] = useState(false);
    if (row.avatar_url && !broken) {
        return (
            // 外部 URL 無法經 next/image 最佳化
            // eslint-disable-next-line @next/next/no-img-element
            <img src={row.avatar_url} alt="" loading="lazy" referrerPolicy="no-referrer"
                onError={() => setBroken(true)}
                className="w-7 h-7 shrink-0 rounded-full object-cover" />
        );
    }
    return (
        <span className="w-7 h-7 shrink-0 rounded-full bg-primary-100 dark:bg-primary-900 text-primary-600 dark:text-primary-300 flex items-center justify-center text-xs font-semibold">
            {row.name.charAt(0)}
        </span>
    );
}

export function LeaderboardCard({ board, period, loading, error, isMember, onPeriod, t }: {
    board: VocabLeaderboard; period: VocabLeaderboardPeriod; loading: boolean; error: boolean;
    isMember: boolean; onPeriod: (p: VocabLeaderboardPeriod) => void; t: T;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow-sm flex flex-col gap-2">
            <div className="flex items-center justify-between px-2">
                <h2 className="font-bold flex items-center gap-1">
                    <Trophy size={18} className="text-primary-500" aria-hidden />{t("leaderboard")}
                </h2>
                <div className="flex gap-1">
                    {PERIODS.map(p => (
                        <button key={p} onClick={() => onPeriod(p)} disabled={loading}
                            aria-pressed={period === p}
                            className={`px-3 py-1 rounded-full text-xs border transition-colors ${period === p
                                ? "border-primary-500 bg-primary-500 text-white"
                                : "border-neutral-200 dark:border-neutral-600 hover:border-primary-400"}`}>
                            {t(PERIOD_KEYS[p])}
                        </button>
                    ))}
                </div>
            </div>
            {loading ? (
                <div className="flex justify-center py-6">
                    <Loader2 size={20} className="animate-spin text-neutral-400" />
                </div>
            ) : error ? (
                <p className="text-center text-sm text-red-500 py-4">{t("lbError")}</p>
            ) : board.top.length === 0 ? (
                <p className="text-center text-sm text-neutral-500 dark:text-neutral-400 py-4">{t("lbEmpty")}</p>
            ) : (
                <div className="flex flex-col divide-y divide-neutral-100 dark:divide-neutral-700">
                    {board.top.map(row => (
                        <div key={`${row.rank}|${row.name}`} className="flex items-center gap-3 py-2 px-2">
                            <span className={`w-7 h-7 shrink-0 rounded-full flex items-center justify-center text-xs font-bold ${row.rank <= 3
                                ? "bg-primary-500 text-white"
                                : "bg-neutral-100 dark:bg-neutral-700 text-neutral-500 dark:text-neutral-400"}`}>
                                {row.rank}
                            </span>
                            <Avatar row={row} />
                            <span className="flex-1 min-w-0 truncate font-medium">{row.name}</span>
                            <span className="text-xs text-neutral-400 dark:text-neutral-500 shrink-0">{t("lbRuns", { count: row.runs })}</span>
                            <span className="text-sm font-semibold text-primary-600 dark:text-primary-400 shrink-0">{row.exp} EXP</span>
                        </div>
                    ))}
                </div>
            )}
            {!loading && !error && isMember && (
                <p className="text-center text-xs text-neutral-500 dark:text-neutral-400 border-t border-neutral-100 dark:border-neutral-700 pt-2">
                    {board.me
                        ? t("lbMyRank", { rank: board.me.rank, exp: board.me.exp })
                        : t("lbNotRanked")}
                </p>
            )}
        </div>
    );
}
