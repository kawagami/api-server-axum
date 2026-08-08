"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Cpu, MemoryStick, HardDrive, Activity, Loader2 } from "lucide-react";
import { getSystemMetrics } from "@/api/metrics";
import type { SystemMetric } from "@/types";
import PageHeader from "@/components/admin/page-header";
import usePolling from "@/hooks/usePolling";
import { ADMIN_LOCALE, ADMIN_TIME_ZONE } from "@/libs/admin-datetime";
import MetricsTrendChart, { type TimeRange } from "./metrics-trend-chart";
import MetricsAuditPanel from "./metrics-audit-panel";

const HOUR_OPTIONS = [24, 72, 168];
const POLL_MS = 60_000; // 後端定時採樣，每分鐘輪詢拉最新

function pct(v: number) {
    return `${v.toFixed(1)}%`;
}

// MB → GB（顯示用），預設保留 1 位小數
function gb(mb: number, digits = 1) {
    return (mb / 1024).toFixed(digits);
}

// 卡片只需要「月/日 時:分」，比 admin-datetime 的完整格式短；locale / 時區沿用同一組常數
const snapshotFmt = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    timeZone: ADMIN_TIME_ZONE,
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
});

function fmtSnapshotTime(iso: string) {
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return iso;
    return snapshotFmt.format(d);
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
        <div className="flex flex-col gap-2 p-5 bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700">
            <div className="flex items-center gap-2 text-neutral-500 dark:text-neutral-400">
                <Icon size={16} />
                <span className="text-sm">{label}</span>
            </div>
            <div className="text-3xl font-bold text-neutral-800 dark:text-neutral-100 tabular-nums">
                {value}
            </div>
            <div className="text-xs text-neutral-400 dark:text-neutral-500 min-h-4">{hint}</div>
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
        <section className="bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700 p-5">
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
    canReadAudit,
}: {
    initial: SystemMetric[];
    initialHours: number;
    /** 有 audit:read 才啟用「拖曳選區間 → 看操作紀錄」 */
    canReadAudit: boolean;
}) {
    const [hours, setHours] = useState(initialHours);
    const [metrics, setMetrics] = useState(initial);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [range, setRange] = useState<TimeRange | null>(null);
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
            // 換範圍等於換資料集，舊選區的時間戳可能已不在圖上
            setRange(null);
            setLoading(true);
            await refresh(h);
            setLoading(false);
        },
        [refresh],
    );

    // 每分鐘輪詢當前範圍（拉最新採樣）；分頁在背景時不打，切回來若已過期就補一次
    usePolling(() => refresh(hoursRef.current), POLL_MS);

    const latest = metrics.length > 0 ? metrics[metrics.length - 1] : null;
    // 四張圖共用同一組選區狀態：在任一張圖拖曳，其他張同步顯示灰帶
    const rangeProps = { range, onRangeChange: canReadAudit ? setRange : undefined };

    // 後端依查詢範圍把資料聚成時間桶（見 repositories/system_metrics.rs 的 bucket_seconds），
    // 每點的 created_at 是桶的起點。桶寬取相鄰採樣點的最小間隔推回來，
    // 用來把選區的結束時間補滿最後一個桶，否則會漏掉桶內的紀錄。
    const bucketMs = useMemo(() => {
        let min = Infinity;
        for (let i = 1; i < metrics.length; i++) {
            const d =
                new Date(metrics[i].created_at).getTime() -
                new Date(metrics[i - 1].created_at).getTime();
            if (d > 0 && d < min) min = d;
        }
        return Number.isFinite(min) ? min : 60_000;
    }, [metrics]);
    const bucketMinutes = Math.round(bucketMs / 60_000);

    return (
        <div className="flex flex-col gap-6">
            <PageHeader
                title="系統指標"
                actions={
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
                }
            />

            {error && <p className="text-sm text-red-600 dark:text-red-400">{error}</p>}

            {latest === null ? (
                <p className="text-center text-neutral-400 dark:text-neutral-500 text-sm py-16 bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700">
                    近 {hours} 小時尚無採樣資料
                </p>
            ) : (
                <>
                    <section className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                        <SnapshotCard
                            icon={Cpu}
                            label="CPU 使用率"
                            value={pct(latest.cpu_pct)}
                            // 有聚合時卡片值是該桶的峰值而非單一採樣，標示清楚免得被當成即時值。
                            // steal 只在真的被抽走時才顯示（一般 VPS 是 0，平時不佔版面）
                            hint={
                                (bucketMinutes > 1
                                    ? `最近 ${bucketMinutes} 分鐘峰值：${fmtSnapshotTime(latest.created_at)}`
                                    : `最新採樣：${fmtSnapshotTime(latest.created_at)}`) +
                                (latest.cpu_steal_pct >= 1
                                    ? `・steal ${pct(latest.cpu_steal_pct)}`
                                    : "")
                            }
                        />
                        <SnapshotCard
                            icon={MemoryStick}
                            label="記憶體"
                            value={`${gb(latest.mem_used_mb, 2)} / ${gb(latest.mem_total_mb, 2)} GB`}
                            hint={`${pct((latest.mem_used_mb / Math.max(1, latest.mem_total_mb)) * 100)} 已用・backend ${latest.backend_rss_mb} MB`}
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
                            {...rangeProps}
                            yMax={100}
                            points={metrics.map(m => ({ t: m.created_at, v: m.cpu_pct }))}
                            format={v => `${Math.round(v)}%`}
                        />
                    </ChartSection>

                    <ChartSection title="記憶體使用量（GB）" loading={loading}>
                        <MetricsTrendChart
                            title="記憶體使用量趨勢"
                            {...rangeProps}
                            yMax={latest.mem_total_mb / 1024}
                            points={metrics.map(m => ({ t: m.created_at, v: m.mem_used_mb / 1024 }))}
                            format={v => `${v.toFixed(2)}`}
                        />
                    </ChartSection>

                    <ChartSection title="Backend 常駐記憶體（MB）" loading={loading}>
                        <MetricsTrendChart
                            title="Backend RSS 趨勢"
                            {...rangeProps}
                            points={metrics.map(m => ({ t: m.created_at, v: m.backend_rss_mb }))}
                            format={v => `${Math.round(v)}`}
                        />
                    </ChartSection>

                    <ChartSection title="Load（1 分鐘平均）" loading={loading}>
                        <MetricsTrendChart
                            title="Load 趨勢"
                            {...rangeProps}
                            points={metrics.map(m => ({ t: m.created_at, v: m.load1 }))}
                            format={v => v.toFixed(2)}
                        />
                    </ChartSection>

                    {canReadAudit && range && (
                        <MetricsAuditPanel
                            from={range.from}
                            to={new Date(new Date(range.to).getTime() + bucketMs).toISOString()}
                            onClear={() => setRange(null)}
                        />
                    )}

                    <p className="text-xs text-neutral-400 dark:text-neutral-500">
                        資料由後端定時採樣，時間以台北時區顯示；此頁每分鐘自動拉取最新採樣。
                        CPU 為整個採樣間隔（1 分鐘）的平均，不含 hypervisor 抽走的 steal。
                        {bucketMinutes > 1 && `範圍較長時會聚合顯示，圖上每點為 ${bucketMinutes} 分鐘內最忙的那一分鐘。`}
                        {canReadAudit && "在任一張圖上橫向拖曳可選取時間區間，下方會列出該區間的後台操作紀錄。"}
                    </p>
                </>
            )}
        </div>
    );
}
