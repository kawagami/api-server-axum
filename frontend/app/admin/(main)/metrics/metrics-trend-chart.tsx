"use client";

import { useId, useRef, useState } from "react";

export interface MetricPoint {
    // ISO 時間字串（UTC），已由呼叫端排序成舊→新
    t: string;
    v: number;
}

/** 選取的時間區間，端點取自資料點的 `t`（四張圖共用同一組時間戳，故可用字串比對定位） */
export interface TimeRange {
    from: string;
    to: string;
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
    // 目前選取區間（由呼叫端集中管理，四張圖同步顯示同一段灰帶）
    range = null,
    // 給了才啟用拖曳選區；單擊（沒拖）視為清除，回傳 null
    onRangeChange,
}: {
    title: string;
    points: MetricPoint[];
    yMax?: number;
    format?: (v: number) => string;
    range?: TimeRange | null;
    onRangeChange?: (range: TimeRange | null) => void;
}) {
    const gradId = useId();
    const svgRef = useRef<SVGSVGElement>(null);
    // 觸控裝置沒有 hover，改點擊/拖曳選取資料點顯示數值
    const [selected, setSelected] = useState<number | null>(null);
    // 拖曳中的暫時區間（放開才提交給呼叫端）
    const [drag, setDrag] = useState<{ start: number; end: number } | null>(null);
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

    // 依 pointer X 座標換算最近的資料點 index
    function idxAt(e: React.PointerEvent<SVGSVGElement>): number | null {
        const svg = svgRef.current;
        if (!svg) return null;
        const rect = svg.getBoundingClientRect();
        const px = ((e.clientX - rect.left) / rect.width) * W;
        const i = rows.length === 1 ? 0 : Math.round(((px - padL) / innerW) * (rows.length - 1));
        return Math.max(0, Math.min(rows.length - 1, i));
    }

    function handleDown(e: React.PointerEvent<SVGSVGElement>) {
        const i = idxAt(e);
        if (i === null) return;
        setSelected(i);
        if (!onRangeChange) return;
        // 抓住 pointer，拖出 svg 外放開也收得到 pointerup
        e.currentTarget.setPointerCapture(e.pointerId);
        setDrag({ start: i, end: i });
    }

    function handleMove(e: React.PointerEvent<SVGSVGElement>) {
        if (!e.buttons) return;
        const i = idxAt(e);
        if (i === null) return;
        setSelected(i);
        setDrag(d => (d ? { ...d, end: i } : d));
    }

    function handleUp() {
        if (!drag) return;
        const [a, b] = [Math.min(drag.start, drag.end), Math.max(drag.start, drag.end)];
        setDrag(null);
        // 單擊（起訖同一點）只是看數值，不動選區——觸控裝置讀數值只能靠點擊，
        // 若順手清掉選區太容易誤觸；要清改按面板上的「清除選取」
        if (a !== b) onRangeChange?.({ from: rows[a].t, to: rows[b].t });
    }

    // 灰帶範圍：拖曳中用暫時值，否則用呼叫端傳入的區間（時間戳對不到就不畫）
    const band = (() => {
        if (drag) return [Math.min(drag.start, drag.end), Math.max(drag.start, drag.end)] as const;
        if (!range) return null;
        const a = rows.findIndex(r => r.t === range.from);
        const b = rows.findIndex(r => r.t === range.to);
        return a >= 0 && b >= 0 ? ([a, b] as const) : null;
    })();

    return (
        <div className="overflow-x-auto">
            <svg
                ref={svgRef}
                viewBox={`0 0 ${W} ${H}`}
                className="w-full min-w-[480px] h-auto select-none cursor-crosshair"
                // 橫向手勢留給選區拖曳，直向仍可捲頁
                style={onRangeChange ? { touchAction: 'pan-y' } : undefined}
                role="img"
                aria-label={title}
                onPointerDown={handleDown}
                onPointerMove={handleMove}
                onPointerUp={handleUp}
                onPointerCancel={() => setDrag(null)}
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

                {/* 選取區間灰帶（四張圖同步） */}
                {band && (
                    <rect
                        x={x(band[0])}
                        y={padT}
                        width={Math.max(2, x(band[1]) - x(band[0]))}
                        height={innerH}
                        fill="rgb(var(--primary-500))"
                        fillOpacity={0.12}
                        stroke="rgb(var(--primary-400))"
                        strokeWidth={1}
                        pointerEvents="none"
                    />
                )}

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

                {/* 資料點：點多到擠成一片就不畫（r=0 的圓連 title 都點不到，純粹是 DOM 負擔） */}
                {rows.length <= 45 &&
                    rows.map((r, i) => (
                        <circle
                            key={`${r.t}-${i}`}
                            cx={x(i)}
                            cy={y(r.v)}
                            r={2.5}
                            fill="rgb(var(--primary-600))"
                        >
                            <title>{`${fmtTooltipTime(r.t)}：${format(r.v)}`}</title>
                        </circle>
                    ))}

                {/* X 軸時間標籤（固定約 6 個） */}
                {rows.map((r, i) =>
                    i % labelStep === 0 ? (
                        <text
                            key={`${r.t}-${i}`}
                            x={x(i)}
                            y={H - 8}
                            textAnchor="middle"
                            className="fill-neutral-400 dark:fill-neutral-500 text-[10px] tabular-nums"
                        >
                            {fmtAxisTime(r.t)}
                        </text>
                    ) : null,
                )}

                {/* 選取點：垂直參考線 + 數值標籤（觸控可用） */}
                {selected !== null && rows[selected] && (() => {
                    const r = rows[selected];
                    const cx = x(selected);
                    const anchor = cx > W - 130 ? "end" : cx < padL + 110 ? "start" : "middle";
                    return (
                        <g pointerEvents="none">
                            <line
                                x1={cx} y1={padT} x2={cx} y2={padT + innerH}
                                stroke="rgb(var(--primary-400))" strokeDasharray="3 3" strokeWidth={1}
                            />
                            <circle
                                cx={cx} cy={y(r.v)} r={4}
                                fill="rgb(var(--primary-600))"
                                className="stroke-white dark:stroke-neutral-900" strokeWidth={1.5}
                            />
                            <text
                                x={cx} y={padT + 10} textAnchor={anchor}
                                className="fill-neutral-600 dark:fill-neutral-300 text-[11px] font-medium tabular-nums"
                            >
                                {`${fmtTooltipTime(r.t)}：${format(r.v)}`}
                            </text>
                        </g>
                    );
                })()}
            </svg>
        </div>
    );
}
