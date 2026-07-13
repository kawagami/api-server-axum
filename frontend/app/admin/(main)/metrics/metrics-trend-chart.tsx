"use client";

import { useId } from "react";

export interface MetricPoint {
    // ISO 時間字串（UTC），已由呼叫端排序成舊→新
    t: string;
    v: number;
}

function fmtAxisTime(iso: string) {
    // 台北時區 HH:MM
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return "";
    return new Intl.DateTimeFormat("zh-TW", {
        timeZone: "Asia/Taipei",
        hour: "2-digit",
        minute: "2-digit",
        hour12: false,
    }).format(d);
}

function fmtTooltipTime(iso: string) {
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

// 純 SVG 折線圖：仿 visitor-trend-chart，單一序列，X 軸時間、Y 軸值
export default function MetricsTrendChart({
    title,
    points,
    // 固定 Y 軸上限（如 CPU% 給 100）；省略則用資料最大值
    yMax,
    // 數值格式化（軸刻度 + tooltip），預設整數千分位
    format = v => Math.round(v).toLocaleString(),
}: {
    title: string;
    points: MetricPoint[];
    yMax?: number;
    format?: (v: number) => string;
}) {
    const gradId = useId();
    const rows = [...points].sort((a, b) => a.t.localeCompare(b.t));

    if (rows.length === 0) {
        return (
            <p className="text-center text-neutral-400 dark:text-neutral-500 text-sm py-12">
                尚無資料
            </p>
        );
    }

    const W = 720;
    const H = 220;
    const padL = 52;
    const padR = 16;
    const padT = 16;
    const padB = 28;
    const innerW = W - padL - padR;
    const innerH = H - padT - padB;

    const max = yMax ?? Math.max(1, ...rows.map(r => r.v));
    const x = (i: number) =>
        rows.length === 1 ? padL + innerW / 2 : padL + (i / (rows.length - 1)) * innerW;
    const y = (v: number) => padT + innerH - (Math.min(v, max) / max) * innerH;

    const pts = rows.map((r, i) => [x(i), y(r.v)] as const);
    const linePath = pts.map((p, i) => `${i === 0 ? "M" : "L"} ${p[0]} ${p[1]}`).join(" ");
    const areaPath = `${linePath} L ${pts[pts.length - 1][0]} ${padT + innerH} L ${pts[0][0]} ${padT + innerH} Z`;

    const yTicks = [0, max / 2, max];
    const labelStep = Math.max(1, Math.ceil(rows.length / 6));

    return (
        <div className="overflow-x-auto">
            <svg
                viewBox={`0 0 ${W} ${H}`}
                className="w-full min-w-[480px] h-auto"
                role="img"
                aria-label={title}
            >
                <defs>
                    <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="rgb(var(--primary-500))" stopOpacity="0.25" />
                        <stop offset="100%" stopColor="rgb(var(--primary-500))" stopOpacity="0" />
                    </linearGradient>
                </defs>

                {/* 格線 + Y 軸刻度（0、½、max） */}
                {yTicks.map((v, i) => (
                    <g key={i}>
                        <line
                            x1={padL}
                            y1={y(v)}
                            x2={W - padR}
                            y2={y(v)}
                            className="stroke-neutral-200 dark:stroke-neutral-700"
                            strokeWidth={1}
                        />
                        <text
                            x={padL - 8}
                            y={y(v) + 3}
                            textAnchor="end"
                            className="fill-neutral-400 dark:fill-neutral-500 text-[10px] tabular-nums"
                        >
                            {format(v)}
                        </text>
                    </g>
                ))}

                {/* 面積 + 折線 */}
                <path d={areaPath} fill={`url(#${gradId})`} />
                <path
                    d={linePath}
                    fill="none"
                    stroke="rgb(var(--primary-500))"
                    strokeWidth={2}
                    strokeLinejoin="round"
                    strokeLinecap="round"
                />

                {/* 資料點 + X 軸時間標籤 */}
                {rows.map((r, i) => (
                    <g key={`${r.t}-${i}`}>
                        <circle
                            cx={x(i)}
                            cy={y(r.v)}
                            r={rows.length > 45 ? 0 : 2.5}
                            fill="rgb(var(--primary-600))"
                        >
                            <title>{`${fmtTooltipTime(r.t)}：${format(r.v)}`}</title>
                        </circle>
                        {i % labelStep === 0 && (
                            <text
                                x={x(i)}
                                y={H - 8}
                                textAnchor="middle"
                                className="fill-neutral-400 dark:fill-neutral-500 text-[10px] tabular-nums"
                            >
                                {fmtAxisTime(r.t)}
                            </text>
                        )}
                    </g>
                ))}
            </svg>
        </div>
    );
}
