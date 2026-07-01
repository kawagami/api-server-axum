"use client";

import { useTranslations } from "next-intl";
import type { LedgerMonthly } from "@/types";

// 圓餅/長條圖皆走 primary 色階 var（跟隨主題）；收入/支出層級用語意綠/紅
const PIE_SHADES = [500, 300, 700, 400, 800, 200, 600, 900, 100, 950];
const pieColor = (i: number) => `rgb(var(--primary-${PIE_SHADES[i % PIE_SHADES.length]}))`;

function fmt(n: number) {
    return n.toLocaleString(undefined, { maximumFractionDigits: 0 });
}

// 極座標 → 直角座標（12 點鐘為 0 度）
function polar(cx: number, cy: number, r: number, deg: number): [number, number] {
    const a = ((deg - 90) * Math.PI) / 180;
    return [cx + r * Math.cos(a), cy + r * Math.sin(a)];
}

function donutSlice(cx: number, cy: number, rOuter: number, rInner: number, start: number, end: number) {
    const [sx, sy] = polar(cx, cy, rOuter, end);
    const [ex, ey] = polar(cx, cy, rOuter, start);
    const [ix, iy] = polar(cx, cy, rInner, start);
    const [jx, jy] = polar(cx, cy, rInner, end);
    const large = end - start > 180 ? 1 : 0;
    return `M ${sx} ${sy} A ${rOuter} ${rOuter} 0 ${large} 0 ${ex} ${ey} L ${ix} ${iy} A ${rInner} ${rInner} 0 ${large} 1 ${jx} ${jy} Z`;
}

interface Slice {
    label: string;
    value: number;
}

export function CategoryPie({ title, slices }: { title: string; slices: Slice[] }) {
    const t = useTranslations('Ledger');
    const positive = slices.filter(s => s.value > 0);
    const total = positive.reduce((s, x) => s + x.value, 0);

    if (total <= 0) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
                <h3 className="font-semibold text-sm mb-3">{title}</h3>
                <p className="text-center text-neutral-400 dark:text-neutral-500 text-sm py-8">{t('noData')}</p>
            </div>
        );
    }

    const cx = 60, cy = 60, rOuter = 54, rInner = 32;
    // 各 slice 前的累積值（避免 map 內變數重指派）
    const offsets = positive.reduce<number[]>((acc, s) => [...acc, acc[acc.length - 1] + s.value], [0]);
    const arcs = positive.map((s, i) => {
        const start = (offsets[i] / total) * 360;
        const end = (offsets[i + 1] / total) * 360;
        return { ...s, start, end, color: pieColor(i), pct: (s.value / total) * 100 };
    });

    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
            <h3 className="font-semibold text-sm mb-3">{title}</h3>
            <div className="flex items-center gap-4">
                <svg viewBox="0 0 120 120" className="w-28 h-28 shrink-0" role="img" aria-label={title}>
                    {arcs.length === 1 ? (
                        <>
                            <circle cx={cx} cy={cy} r={(rOuter + rInner) / 2} fill="none" stroke={arcs[0].color} strokeWidth={rOuter - rInner} />
                        </>
                    ) : (
                        arcs.map(a => <path key={a.label} d={donutSlice(cx, cy, rOuter, rInner, a.start, a.end)} fill={a.color} />)
                    )}
                </svg>
                <ul className="flex-1 min-w-0 flex flex-col gap-1 text-xs">
                    {arcs.map(a => (
                        <li key={a.label} className="flex items-center gap-2">
                            <span className="w-2.5 h-2.5 rounded-sm shrink-0" style={{ backgroundColor: a.color }} />
                            <span className="truncate flex-1">{a.label}</span>
                            <span className="text-neutral-500 dark:text-neutral-400 tabular-nums">{fmt(a.value)}</span>
                            <span className="text-neutral-400 dark:text-neutral-500 tabular-nums w-12 text-right">{a.pct.toFixed(1)}%</span>
                        </li>
                    ))}
                </ul>
            </div>
        </div>
    );
}

export function MonthlyBars({ monthly }: { monthly: LedgerMonthly[] }) {
    const t = useTranslations('Ledger');
    const rows = monthly.map(m => ({ month: m.month, income: Number(m.income), expense: Number(m.expense) }));
    const max = Math.max(1, ...rows.flatMap(r => [r.income, r.expense]));

    if (rows.length === 0) {
        return (
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
                <h3 className="font-semibold text-sm mb-3">{t('monthlyTrend')}</h3>
                <p className="text-center text-neutral-400 dark:text-neutral-500 text-sm py-8">{t('noData')}</p>
            </div>
        );
    }

    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
            <div className="flex items-center justify-between mb-3">
                <h3 className="font-semibold text-sm">{t('monthlyTrend')}</h3>
                <div className="flex items-center gap-3 text-xs text-neutral-500 dark:text-neutral-400">
                    <span className="flex items-center gap-1"><span className="w-2.5 h-2.5 rounded-sm bg-green-500" />{t('income')}</span>
                    <span className="flex items-center gap-1"><span className="w-2.5 h-2.5 rounded-sm bg-red-500" />{t('expense')}</span>
                </div>
            </div>
            <div className="overflow-x-auto">
                <div className="flex items-end gap-4 h-40 min-w-fit px-1">
                    {rows.map(r => (
                        <div key={r.month} className="flex flex-col items-center gap-1 shrink-0">
                            <div className="flex items-end gap-1 h-32" title={`${t('income')} ${fmt(r.income)} / ${t('expense')} ${fmt(r.expense)}`}>
                                <div className="w-4 rounded-t bg-green-500/90" style={{ height: `${(r.income / max) * 100}%` }} />
                                <div className="w-4 rounded-t bg-red-500/90" style={{ height: `${(r.expense / max) * 100}%` }} />
                            </div>
                            <span className="text-[10px] text-neutral-500 dark:text-neutral-400 whitespace-nowrap">{r.month.slice(2)}</span>
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}
