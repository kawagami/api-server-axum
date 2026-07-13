"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { Cpu, MemoryStick, HardDrive, Activity, Loader2 } from "lucide-react";
import { getSystemMetrics } from "@/api/metrics";
import type { SystemMetric } from "@/types";
import MetricsTrendChart from "./metrics-trend-chart";

const HOUR_OPTIONS = [24, 72, 168];
const POLL_MS = 60_000; // 後端定時採樣，每分鐘輪詢拉最新

function pct(v: number) {
    return `${v.toFixed(1)}%`;
}

// MB → GB（顯示用），保留 1 位小數
function gb(mb: number) {
    return (mb / 1024).toFixed(1);
}

function fmtSnapshotTime(iso: string) {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return new Intl.DateTimeFormat("zh-TW", {
        timeZone: "Asia/Taipei",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
    }).format(d);
}

function SnapshotCard({
    icon: Icon,
    label,
    value,
    hint,
}: {
    icon: typeof Cpu;
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

function ChartSection({
    title,
    loading,
    children,
}: {
    title: string;
    loading: boolean;
    children: React.ReactNode;
}) {
    return (
        <section className="bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700 p-5">
            <div className="flex items-center justify-between mb-4">
                <h2 className="font-semibold text-sm text-neutral-700 dark:text-neutral-200">
                    {title}
                </h2>
                {loading && <Loader2 size={16} className="animate-spin text-neutral-400" />}
            </div>
            {children}
        </section>
    );
}

export default function MetricsView({
    initial,
    initialHours,
}: {
    initial: SystemMetric[];
    initialHours: number;
}) {
    const [hours, setHours] = useState(initialHours);
    const [metrics, setMetrics] = useState(initial);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    // 避免輪詢回應覆蓋掉使用者剛切換的時間範圍
    const hoursRef = useRef(hours);
    useEffect(() => {
        hoursRef.current = hours;
    }, [hours]);

    const refresh = useCallback(async (h: number) => {
        try {
            const next = await getSystemMetrics(h);
            // 回應期間範圍又被切換 → 丟棄這次結果
            if (hoursRef.current === h) {
                setMetrics(next);
                setError(null);
            }
        } catch {
            setError("讀取失敗，稍後重試");
        }
    }, []);

    // 切換時間範圍：重新抓取
    const onPickHours = useCallback(
        async (h: number) => {
            if (h === hoursRef.current) return;
            setHours(h);
            setLoading(true);
            await refresh(h);
            setLoading(false);
        },
        [refresh],
    );

    // 每分鐘輪詢當前範圍（拉最新採樣）
    useEffect(() => {
        const id = setInterval(() => refresh(hoursRef.current), POLL_MS);
        return () => clearInterval(id);
    }, [refresh]);

    const latest = metrics.length > 0 ? metrics[metrics.length - 1] : null;

    return (
        <div className="max-w-5xl mx-auto flex flex-col gap-6">
            <div className="flex flex-wrap items-center justify-between gap-3">
                <h1 className="text-2xl font-bold text-neutral-800 dark:text-neutral-100">系統指標</h1>
                <div className="inline-flex rounded-lg border border-neutral-200 dark:border-neutral-700 overflow-hidden">
                    {HOUR_OPTIONS.map(h => (
                        <button
                            key={h}
                            type="button"
                            onClick={() => onPickHours(h)}
                            className={`px-4 py-1.5 text-sm font-medium transition-colors ${
                                h === hours
                                    ? "bg-primary-600 text-white"
                                    : "bg-white dark:bg-neutral-900 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-50 dark:hover:bg-neutral-800"
                            }`}
                        >
                            {h} 小時
                        </button>
                    ))}
                </div>
            </div>

            {error && <p className="text-sm text-red-600 dark:text-red-400">{error}</p>}

            {latest === null ? (
                <p className="text-center text-neutral-400 dark:text-neutral-500 text-sm py-16 bg-white dark:bg-neutral-900 rounded-lg shadow border border-neutral-200 dark:border-neutral-700">
                    近 {hours} 小時尚無採樣資料
                </p>
            ) : (
                <>
                    <section className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                        <SnapshotCard
                            icon={Cpu}
                            label="CPU 使用率"
                            value={pct(latest.cpu_pct)}
                            hint={`最新採樣：${fmtSnapshotTime(latest.created_at)}`}
                        />
                        <SnapshotCard
                            icon={MemoryStick}
                            label="記憶體"
                            value={`${gb(latest.mem_used_mb)} / ${gb(latest.mem_total_mb)} GB`}
                            hint={`${pct((latest.mem_used_mb / Math.max(1, latest.mem_total_mb)) * 100)} 已用`}
                        />
                        <SnapshotCard
                            icon={HardDrive}
                            label="磁碟"
                            value={`${gb(latest.disk_used_mb)} / ${gb(latest.disk_total_mb)} GB`}
                            hint={`${pct((latest.disk_used_mb / Math.max(1, latest.disk_total_mb)) * 100)} 已用`}
                        />
                        <SnapshotCard
                            icon={Activity}
                            label="Load（1／5／15 分）"
                            value={`${latest.load1.toFixed(2)}`}
                            hint={`5 分 ${latest.load5.toFixed(2)}・15 分 ${latest.load15.toFixed(2)}`}
                        />
                    </section>

                    <ChartSection title="CPU 使用率（%）" loading={loading}>
                        <MetricsTrendChart
                            title="CPU 使用率趨勢"
                            yMax={100}
                            points={metrics.map(m => ({ t: m.created_at, v: m.cpu_pct }))}
                            format={v => `${Math.round(v)}%`}
                        />
                    </ChartSection>

                    <ChartSection title="記憶體使用量（GB）" loading={loading}>
                        <MetricsTrendChart
                            title="記憶體使用量趨勢"
                            yMax={latest.mem_total_mb / 1024}
                            points={metrics.map(m => ({ t: m.created_at, v: m.mem_used_mb / 1024 }))}
                            format={v => `${v.toFixed(1)}`}
                        />
                    </ChartSection>

                    <ChartSection title="Load（1 分鐘平均）" loading={loading}>
                        <MetricsTrendChart
                            title="Load 趨勢"
                            points={metrics.map(m => ({ t: m.created_at, v: m.load1 }))}
                            format={v => v.toFixed(2)}
                        />
                    </ChartSection>

                    <p className="text-xs text-neutral-400 dark:text-neutral-500">
                        資料由後端定時採樣，時間以台北時區顯示；此頁每分鐘自動拉取最新採樣。
                    </p>
                </>
            )}
        </div>
    );
}
