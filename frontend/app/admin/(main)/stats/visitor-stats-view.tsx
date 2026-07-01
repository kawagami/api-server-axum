"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Users, CalendarRange, Loader2 } from "lucide-react";
import { getVisitorStats } from "@/api/stats";
import type { VisitorStats } from "@/types";
import VisitorTrendChart from "./visitor-trend-chart";

const DAY_OPTIONS = [7, 30, 90];
const POLL_MS = 60_000; // 今日數字隨 HLL 累積，每分鐘輪詢更新

function fmt(n: number) {
    return n.toLocaleString();
}

function StatCard({
    icon: Icon,
    label,
    value,
    hint,
}: {
    icon: typeof Users;
    label: string;
    value: string;
    hint: string;
}) {
    return (
        <div className="flex flex-col gap-2 p-5 bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700">
            <div className="flex items-center gap-2 text-neutral-500 dark:text-neutral-400">
                <Icon size={16} />
                <span className="text-sm">{label}</span>
            </div>
            <div className="text-3xl font-bold text-neutral-800 dark:text-neutral-100 tabular-nums">
                {value}
            </div>
            <div className="text-xs text-neutral-400 dark:text-neutral-500 min-h-[1rem]">{hint}</div>
        </div>
    );
}

export default function VisitorStatsView({
    initial,
    initialDays,
}: {
    initial: VisitorStats;
    initialDays: number;
}) {
    const [days, setDays] = useState(initialDays);
    const [stats, setStats] = useState(initial);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    // 避免輪詢回應覆蓋掉使用者剛切換的天數
    const daysRef = useRef(days);
    useEffect(() => {
        daysRef.current = days;
    }, [days]);

    const refresh = useCallback(async (d: number) => {
        try {
            const next = await getVisitorStats(d);
            // 回應期間天數又被切換 → 丟棄這次結果
            if (daysRef.current === d) {
                setStats(next);
                setError(null);
            }
        } catch {
            setError("讀取失敗，稍後重試");
        }
    }, []);

    // 切換天數：重新抓取
    const onPickDays = useCallback(
        async (d: number) => {
            if (d === daysRef.current) return;
            setDays(d);
            setLoading(true);
            await refresh(d);
            setLoading(false);
        },
        [refresh],
    );

    // 每分鐘輪詢當前天數（更新今日累積數字）
    useEffect(() => {
        const id = setInterval(() => refresh(daysRef.current), POLL_MS);
        return () => clearInterval(id);
    }, [refresh]);

    return (
        <div className="max-w-5xl mx-auto flex flex-col gap-6">
            <div className="flex flex-wrap items-center justify-between gap-3">
                <h1 className="text-2xl font-bold text-neutral-800 dark:text-neutral-100">到訪統計</h1>
                <div className="inline-flex rounded-lg border border-neutral-200 dark:border-neutral-700 overflow-hidden">
                    {DAY_OPTIONS.map(d => (
                        <button
                            key={d}
                            type="button"
                            onClick={() => onPickDays(d)}
                            className={`px-4 py-1.5 text-sm font-medium transition-colors ${
                                d === days
                                    ? "bg-primary-600 text-white"
                                    : "bg-white dark:bg-neutral-900 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-800"
                            }`}
                        >
                            {d} 天
                        </button>
                    ))}
                </div>
            </div>

            {error && (
                <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
            )}

            <section className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <StatCard
                    icon={Users}
                    label="今日不重複到訪"
                    value={fmt(stats.today.unique_visitors)}
                    hint={`${stats.today.date}・即時，每分鐘更新（約 ±0.8% 誤差）`}
                />
                <StatCard
                    icon={CalendarRange}
                    label={`近 ${days} 天不重複到訪`}
                    value={fmt(stats.last_n_days_unique)}
                    hint="跨日去重（同一人多天只計一次），非每日相加"
                />
            </section>

            <section className="bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700 p-5">
                <div className="flex items-center justify-between mb-4">
                    <h2 className="font-semibold text-sm text-neutral-700 dark:text-neutral-200">
                        每日不重複到訪趨勢
                    </h2>
                    {loading && <Loader2 size={16} className="animate-spin text-neutral-400" />}
                </div>
                <VisitorTrendChart history={stats.history} />
            </section>

            <p className="text-xs text-neutral-400 dark:text-neutral-500">
                今日數字來自 Redis 即時 HLL（會隨時間增長、約 ±0.8% 誤差）；昨日以前為資料庫落地值。
                「近 {days} 天不重複」為跨日去重，必定 ≤ 每日數字相加。
            </p>
        </div>
    );
}
