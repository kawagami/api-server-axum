"use client";

import { useState, useEffect } from "react";
import { useRouter } from "@/i18n/navigation";
import { useTranslations } from "next-intl";
import { Plus, Pencil, Trash2, BarChart2 } from "lucide-react";
import { postPortfolio } from "@/api/portfolio";
import { putPortfolio } from "@/api/portfolio";
import { deletePortfolio } from "@/api/portfolio";
import PortfolioForm from "./portfolio-form";
import StockHistoryTable from "./stock-history-table";
import type { PortfolioSummaryEntry, PortfolioEntryInput } from "@/types";

interface Props {
    initialEntries: PortfolioSummaryEntry[];
}

type Mode =
    | { type: 'list' }
    | { type: 'add' }
    | { type: 'edit'; entry: PortfolioSummaryEntry }
    | { type: 'history'; entry: PortfolioSummaryEntry };

function SummaryCard({ label, value, sub, colored, positive }: {
    label: string;
    value: string;
    sub?: string;
    colored?: boolean;
    positive?: boolean;
}) {
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow-sm border dark:border-neutral-700">
            <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-1">{label}</p>
            <p className={`font-semibold text-lg ${colored ? (positive ? 'text-red-500' : 'text-green-500') : ''}`}>
                {value}
                {sub && <span className="font-normal text-xs ml-1">{sub}</span>}
            </p>
        </div>
    );
}

const fmtInt = (n: number) => n.toLocaleString(undefined, { maximumFractionDigits: 0 });
const signed = (n: number) => `${n >= 0 ? '+' : ''}${fmtInt(n)}`;
const signedPct = (n: number) => `${n >= 0 ? '+' : ''}${n.toFixed(2)}%`;

export default function PortfolioClient({ initialEntries }: Props) {
    const t = useTranslations('Portfolio');
    const router = useRouter();
    const [entries, setEntries] = useState<PortfolioSummaryEntry[]>(initialEntries);
    const [mode, setMode] = useState<Mode>({ type: 'list' });
    const [mutating, setMutating] = useState(false);

    // router.refresh() 後同步新 props（adjust-state-during-render 模式，取代 useEffect）
    const [prevInitialEntries, setPrevInitialEntries] = useState(initialEntries);
    if (prevInitialEntries !== initialEntries) {
        setPrevInitialEntries(initialEntries);
        setEntries(initialEntries);
    }

    async function handleAdd(input: PortfolioEntryInput) {
        await postPortfolio(input);
        router.refresh();
        setMode({ type: 'list' });
    }

    async function handleEdit(entry: PortfolioSummaryEntry, input: PortfolioEntryInput) {
        await putPortfolio(entry.id, input);
        router.refresh();
        setMode({ type: 'list' });
    }

    async function handleDelete(id: string) {
        if (!confirm(t('confirmDelete'))) return;
        setMutating(true);
        try {
            await deletePortfolio(id);
            setEntries(prev => prev.filter(e => e.id !== id));
            router.refresh();
        } finally {
            setMutating(false);
        }
    }

    // Summary totals
    const totalCost = entries.reduce((s, e) => s + e.cost_per_share * e.shares, 0);
    const pricedEntries = entries.filter(e => e.current_value !== null);
    const totalValue = pricedEntries.reduce((s, e) => s + (e.current_value ?? 0), 0);
    const totalPnl = pricedEntries.reduce((s, e) => s + (e.pnl ?? 0), 0);
    const totalPnlPct = totalCost > 0 ? (totalPnl / totalCost) * 100 : 0;
    const hasPrices = pricedEntries.length > 0;

    // 今日增減：只加總拿得到前一交易日行情的持股（新加入的持股可能只有一天資料）。
    // 百分比的分母是「這些持股的前收市值」而不是總成本 —— 問的是「今天漲跌幾 %」，
    // 用成本當分母算出來的是別的東西。
    const dayEntries = entries.filter(e => e.day_value_change !== null);
    const totalDayChange = dayEntries.reduce((s, e) => s + (e.day_value_change ?? 0), 0);
    const prevValue = dayEntries.reduce((s, e) => s + (e.prev_close ?? 0) * e.shares, 0);
    const totalDayChangePct = prevValue > 0 ? (totalDayChange / prevValue) * 100 : 0;
    const hasDayChange = dayEntries.length > 0;
    // 部分持股沒有前一日行情 → 這個數字不是整個投組的當日增減，要標出來
    const dayChangePartial = hasDayChange && dayEntries.length < entries.length;

    return (
        <div className="flex flex-col gap-4">
            {/* Summary bar */}
            {entries.length > 0 && (
                <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3 mb-2">
                    <SummaryCard
                        label={t('totalCost')}
                        value={fmtInt(totalCost)}
                    />
                    <SummaryCard
                        label={t('currentValue')}
                        value={hasPrices ? fmtInt(totalValue) : '-'}
                    />
                    <SummaryCard
                        label={t('pnl')}
                        value={hasPrices ? signed(totalPnl) : '-'}
                        colored={hasPrices}
                        positive={totalPnl >= 0}
                    />
                    <SummaryCard
                        label={t('pnlPercent')}
                        value={hasPrices ? signedPct(totalPnlPct) : '-'}
                        colored={hasPrices}
                        positive={totalPnlPct >= 0}
                    />
                    <SummaryCard
                        label={dayChangePartial ? t('todayChangePartial') : t('todayChange')}
                        value={hasDayChange ? signed(totalDayChange) : '-'}
                        sub={hasDayChange ? `(${signedPct(totalDayChangePct)})` : undefined}
                        colored={hasDayChange}
                        positive={totalDayChange >= 0}
                    />
                </div>
            )}

            {/* Add button */}
            {mode.type === 'list' && (
                <div className="flex justify-end">
                    <button
                        onClick={() => setMode({ type: 'add' })}
                        className="flex items-center gap-2 px-4 py-2 rounded-sm bg-primary-500 text-white hover:bg-primary-600 text-sm"
                    >
                        <Plus size={16} />
                        {t('addEntry')}
                    </button>
                </div>
            )}

            {/* Add form */}
            {mode.type === 'add' && (
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow-sm border dark:border-neutral-700">
                    <h2 className="font-semibold mb-4">{t('addEntry')}</h2>
                    <PortfolioForm
                        onSave={handleAdd}
                        onCancel={() => setMode({ type: 'list' })}
                    />
                </div>
            )}

            {/* Edit form */}
            {mode.type === 'edit' && (
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow-sm border dark:border-neutral-700">
                    <h2 className="font-semibold mb-4">{t('editEntry')}</h2>
                    <PortfolioForm
                        initial={mode.entry}
                        onSave={input => handleEdit(mode.entry, input)}
                        onCancel={() => setMode({ type: 'list' })}
                    />
                </div>
            )}

            {/* Entry list */}
            {entries.length === 0 && mode.type === 'list' ? (
                <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{t('noEntries')}</p>
            ) : (
                <div className="flex flex-col gap-3">
                    {entries.map(entry => {
                        const hasPnl = entry.pnl !== null;
                        return (
                            <div
                                key={entry.id}
                                className="bg-white dark:bg-neutral-800 rounded-xl px-5 py-4 shadow-sm border dark:border-neutral-700 flex items-center justify-between gap-4"
                            >
                                <div className="flex flex-col gap-1 min-w-0 flex-1">
                                    {/* Stock code + name */}
                                    <div className="flex items-baseline gap-2">
                                        <span className="font-bold text-lg">{entry.stock_code}</span>
                                        {entry.stock_name && (
                                            <span className="text-sm text-neutral-500 dark:text-neutral-400">{entry.stock_name}</span>
                                        )}
                                    </div>
                                    {/* Meta */}
                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                        {entry.buy_date} · {entry.shares.toLocaleString()} {t('shares')} · @{entry.cost_per_share}
                                    </span>
                                    {/* P&L */}
                                    <div className="flex items-center gap-3 mt-1 text-sm">
                                        {hasPnl ? (
                                            <>
                                                <span className="text-neutral-500 dark:text-neutral-400">
                                                    {t('currentPrice')}: {entry.current_price?.toFixed(2)}
                                                </span>
                                                {entry.day_change !== null && (
                                                    <span className={`text-xs ${entry.day_change >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                                                        {t('todayChange')} {entry.day_change >= 0 ? '+' : ''}{entry.day_change.toFixed(2)}
                                                        <span className="ml-1">({signedPct(entry.day_change_pct ?? 0)})</span>
                                                    </span>
                                                )}
                                                <span className={`font-semibold ${entry.pnl! >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                                                    {entry.pnl! >= 0 ? '+' : ''}{entry.pnl!.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                                                    <span className="font-normal ml-1 text-xs">
                                                        ({entry.pnl_pct! >= 0 ? '+' : ''}{entry.pnl_pct!.toFixed(2)}%)
                                                    </span>
                                                </span>
                                            </>
                                        ) : (
                                            <span className="text-neutral-400 dark:text-neutral-500 text-xs">{t('noPriceData')}</span>
                                        )}
                                    </div>
                                </div>

                                {/* Actions */}
                                <div className="flex items-center gap-2 shrink-0">
                                    <button
                                        onClick={() => setMode({ type: 'history', entry })}
                                        title={t('viewHistory')}
                                        className="p-2 rounded-sm hover:bg-neutral-100 dark:hover:bg-neutral-700 text-primary-500"
                                    >
                                        <BarChart2 size={16} />
                                    </button>
                                    <button
                                        onClick={() => setMode({ type: 'edit', entry })}
                                        title={t('editEntry')}
                                        className="p-2 rounded-sm hover:bg-neutral-100 dark:hover:bg-neutral-700"
                                    >
                                        <Pencil size={16} />
                                    </button>
                                    <button
                                        onClick={() => handleDelete(entry.id)}
                                        disabled={mutating}
                                        title={t('deleteEntry')}
                                        className="p-2 rounded-sm hover:bg-neutral-100 dark:hover:bg-neutral-700 text-red-500 disabled:opacity-50"
                                    >
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}

            {/* History modal */}
            {mode.type === 'history' && (
                <StockHistoryTable
                    entry={mode.entry}
                    onClose={() => setMode({ type: 'list' })}
                />
            )}
        </div>
    );
}
