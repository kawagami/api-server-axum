"use client";

import { useState } from "react";

export interface SpendBar {
    key: string;
    /** X 軸標籤；空字串 = 這根不標 */
    label: string;
    value: number;
    /** 那段期間完全沒紀錄：畫灰色底標，不畫柱子（和「有記但 0 元」區分） */
    unrecorded?: boolean;
    /** 選取時顯示的完整說明（日期、金額…） */
    detail: string;
}

interface Props {
    bars: SpendBar[];
    ariaLabel: string;
    formatValue: (n: number) => string;
    /** 水平參考線（例如日均） */
    reference?: { value: number; label: string };
    /** 沒選取時讀數列顯示的文字 */
    hint: string;
}

const W = 720;
const H = 220;
const PAD_L = 56;
const PAD_R = 20;
const PAD_T = 12;
const PAD_B = 24;
const INNER_W = W - PAD_L - PAD_R;
const INNER_H = H - PAD_T - PAD_B;
const MAX_BAR_W = 24;
const RADIUS = 4;

/** 往上取到 1 / 2 / 2.5 / 3 / 4 / 5 / 6 / 8 × 10ⁿ，讓 Y 軸刻度（0、½、頂）是整數又不浪費高度 */
function niceCeil(n: number): number {
    if (n <= 0) return 1;
    const mag = 10 ** Math.floor(Math.log10(n));
    const step = [1, 2, 2.5, 3, 4, 5, 6, 8, 10].find(s => s * mag >= n) ?? 10;
    return step * mag;
}

/** 頂端圓角、底部貼齊基線的柱子 */
function barPath(x: number, y: number, w: number, h: number): string {
    const r = Math.min(RADIUS, h, w / 2);
    return `M${x},${y + h} V${y + r} Q${x},${y} ${x + r},${y} H${x + w - r} Q${x + w},${y} ${x + w},${y + r} V${y + h} Z`;
}

/** 純 SVG 柱狀圖（單一序列）。滑鼠移上去 / 點一下顯示該根的說明；整個欄位都是點擊範圍。 */
export default function SpendChart({ bars, ariaLabel, formatValue, reference, hint }: Props) {
    const [selected, setSelected] = useState<number | null>(null);

    const max = niceCeil(Math.max(0, reference?.value ?? 0, ...bars.map(b => b.value)));
    const slot = INNER_W / Math.max(1, bars.length);
    const barW = Math.min(MAX_BAR_W, slot * 0.7);
    const y = (v: number) => PAD_T + INNER_H - (v / max) * INNER_H;
    const baseline = PAD_T + INNER_H;
    const current = selected !== null ? bars[selected] : undefined;

    return (
        <div className="flex flex-col gap-2">
            <p className="text-sm tabular-nums min-h-5 text-neutral-600 dark:text-neutral-300" aria-live="polite">
                {current ? current.detail : <span className="text-neutral-400 dark:text-neutral-500">{hint}</span>}
            </p>
            <div className="overflow-x-auto">
                <svg
                    viewBox={`0 0 ${W} ${H}`}
                    className="w-full min-w-[480px] h-auto select-none"
                    role="img"
                    aria-label={ariaLabel}
                    // 觸控放開時也會觸發 pointerleave，只有滑鼠離開才清掉選取
                    onPointerLeave={e => { if (e.pointerType === "mouse") setSelected(null); }}
                >
                    {[0, max / 2, max].map(v => (
                        <g key={v}>
                            <line
                                x1={PAD_L} x2={W - PAD_R} y1={y(v)} y2={y(v)}
                                className="stroke-neutral-200 dark:stroke-neutral-700"
                                strokeWidth={1}
                            />
                            <text
                                x={PAD_L - 8} y={y(v) + 3} textAnchor="end"
                                className="fill-neutral-400 dark:fill-neutral-500 text-[10px] tabular-nums"
                            >
                                {formatValue(v)}
                            </text>
                        </g>
                    ))}

                    {bars.map((b, i) => {
                        const cx = PAD_L + slot * (i + 0.5);
                        const active = selected === i;
                        return (
                            <g key={b.key}>
                                {b.unrecorded ? (
                                    <rect
                                        x={cx - barW / 2} y={baseline - 2} width={barW} height={2} rx={1}
                                        className="fill-neutral-300 dark:fill-neutral-600"
                                    />
                                ) : (
                                    <path
                                        // 0 元也留 2px，看得出「有記」
                                        d={barPath(cx - barW / 2, Math.min(y(b.value), baseline - 2), barW, Math.max(2, baseline - y(b.value)))}
                                        fill={active ? "rgb(var(--primary-600))" : "rgb(var(--primary-500))"}
                                    />
                                )}
                                {b.label && (
                                    <text
                                        x={cx} y={H - 8} textAnchor="middle"
                                        className="fill-neutral-400 dark:fill-neutral-500 text-[10px] tabular-nums"
                                    >
                                        {b.label}
                                    </text>
                                )}
                                <rect
                                    x={PAD_L + slot * i} y={PAD_T} width={slot} height={INNER_H}
                                    fill="transparent"
                                    className={active ? "fill-neutral-500/10" : undefined}
                                    onPointerEnter={() => setSelected(i)}
                                    onPointerDown={() => setSelected(i)}
                                >
                                    <title>{b.detail}</title>
                                </rect>
                            </g>
                        );
                    })}

                    {reference && reference.value > 0 && (
                        <g pointerEvents="none">
                            <line
                                x1={PAD_L} x2={W - PAD_R} y1={y(reference.value)} y2={y(reference.value)}
                                className="stroke-neutral-500 dark:stroke-neutral-400"
                                strokeWidth={1} strokeDasharray="4 3"
                            />
                            <text
                                x={W - PAD_R} y={y(reference.value) - 4} textAnchor="end"
                                // 底色描邊當光暈：壓到柱子上時字還讀得到
                                paintOrder="stroke" strokeWidth={3} strokeLinejoin="round"
                                className="fill-neutral-500 dark:fill-neutral-400 stroke-white dark:stroke-neutral-800 text-[10px] tabular-nums"
                            >
                                {reference.label}
                            </text>
                        </g>
                    )}
                </svg>
            </div>
        </div>
    );
}
