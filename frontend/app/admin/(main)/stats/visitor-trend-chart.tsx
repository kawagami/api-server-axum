"use client";

import { useId } from "react";
import type { VisitorDayCount } from "@/types";

function fmtAxisDate(date: string) {
    // 'YYYY-MM-DD' → 'MM/DD'，避免時區位移直接切字串
    const [, m, d] = date.split("-");
    return m && d ? `${m}/${d}` : date;
}

// 純 SVG 折線圖：history 一律排序成舊→新再畫
export default function VisitorTrendChart({ history }: { history: VisitorDayCount[] }) {
    const gradId = useId();
    const rows = [...history].sort((a, b) => a.date.localeCompare(b.date));

    if (rows.length === 0) {
        return (
            <p className="text-center text-neutral-400 dark:text-neutral-500 text-sm py-12">
                尚無資料
            </p>
        );
    }

    const W = 720;
    const H = 260;
    const padL = 44;
    const padR = 16;
    const padT = 16;
    const padB = 28;
    const innerW = W - padL - padR;
    const innerH = H - padT - padB;

    const max = Math.max(1, ...rows.map(r => r.unique_visitors));
    const x = (i: number) =>
        rows.length === 1 ? padL + innerW / 2 : padL + (i / (rows.length - 1)) * innerW;
    const y = (v: number) => padT + innerH - (v / max) * innerH;

    const points = rows.map((r, i) => [x(i), y(r.unique_visitors)] as const);
    const linePath = points.map((p, i) => `${i === 0 ? "M" : "L"} ${p[0]} ${p[1]}`).join(" ");
    const areaPath = `${linePath} L ${points[points.length - 1][0]} ${padT + innerH} L ${points[0][0]} ${padT + innerH} Z`;

    // 水平格線 / Y 軸刻度（0、½、max）
    const yTicks = [0, max / 2, max];
    // X 軸最多顯示約 6 個標籤
    const labelStep = Math.max(1, Math.ceil(rows.length / 6));

    return (
        <div className="overflow-x-auto">
            <svg
                viewBox={`0 0 ${W} ${H}`}
                className="w-full min-w-[480px] h-auto"
                role="img"
                aria-label="每日不重複到訪趨勢"
            >
                <defs>
                    <linearGradient id={gradId} x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="rgb(var(--primary-500))" stopOpacity="0.25" />
                        <stop offset="100%" stopColor="rgb(var(--primary-500))" stopOpacity="0" />
                    </linearGradient>
                </defs>

                {/* 格線 + Y 軸刻度 */}
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
                            {Math.round(v).toLocaleString()}
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

                {/* 資料點 + X 軸標籤 */}
                {rows.map((r, i) => (
                    <g key={r.date}>
                        <circle cx={x(i)} cy={y(r.unique_visitors)} r={rows.length > 45 ? 0 : 2.5} fill="rgb(var(--primary-600))">
                            <title>{`${r.date}：${r.unique_visitors.toLocaleString()}`}</title>
                        </circle>
                        {i % labelStep === 0 && (
                            <text
                                x={x(i)}
                                y={H - 8}
                                textAnchor="middle"
                                className="fill-neutral-400 dark:fill-neutral-500 text-[10px] tabular-nums"
                            >
                                {fmtAxisDate(r.date)}
                            </text>
                        )}
                    </g>
                ))}
            </svg>
        </div>
    );
}
