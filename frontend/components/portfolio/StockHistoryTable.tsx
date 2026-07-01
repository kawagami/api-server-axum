"use client";

import { useEffect, useState } from "react";
import { useTranslations } from "next-intl";
import { getPortfolioHistory } from "@/api/portfolio";
import type { HistoryRecord, PortfolioEntry } from "@/types";

interface Props {
    entry: PortfolioEntry;
    onClose: () => void;
}

export default function StockHistoryTable({ entry, onClose }: Props) {
    const t = useTranslations('Portfolio');
    const [records, setRecords] = useState<HistoryRecord[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(false);

    useEffect(() => {
        getPortfolioHistory(entry.id)
            .then(setRecords)
            .catch(() => setError(true))
            .finally(() => setLoading(false));
    }, [entry.id]);

    const totalCost = entry.cost_per_share * entry.shares;
    const latest = records.at(-1) ?? null;
    const currentValue = latest !== null ? latest.close * entry.shares : null;
    const totalPnl = latest?.pnl ?? null;
    const totalPnlPct = latest?.pnl_pct ?? null;

    // daily close-to-close change (not from backend, computed locally)
    function dailyChange(i: number): number {
        if (i === 0) return 0;
        return records[i].close - records[i - 1].close;
    }

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onClick={onClose}>
            <div
                className="bg-white dark:bg-neutral-800 rounded-xl shadow-xl w-full max-w-2xl max-h-[85vh] flex flex-col mx-4"
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between px-5 py-4 border-b dark:border-neutral-700">
                    <div>
                        <h2 className="font-bold text-lg">{entry.stock_code} {t('historyTitle')}</h2>
                        <p className="text-xs text-neutral-500 dark:text-neutral-400">
                            {t('buyDate')}: {entry.buy_date} · {t('costPerShare')}: {entry.cost_per_share} · {t('shares')}: {entry.shares}
                        </p>
                    </div>
                    <button
                        onClick={onClose}
                        className="text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200 text-xl leading-none px-2"
                    >
                        ✕
                    </button>
                </div>

                {/* Summary */}
                {currentValue !== null && totalPnl !== null && (
                    <div className="grid grid-cols-3 gap-3 px-5 py-3 border-b dark:border-neutral-700 text-sm">
                        <div>
                            <p className="text-neutral-500 dark:text-neutral-400">{t('totalCost')}</p>
                            <p className="font-semibold">{totalCost.toLocaleString()}</p>
                        </div>
                        <div>
                            <p className="text-neutral-500 dark:text-neutral-400">{t('currentValue')}</p>
                            <p className="font-semibold">{currentValue.toLocaleString()}</p>
                        </div>
                        <div>
                            <p className="text-neutral-500 dark:text-neutral-400">{t('pnl')}</p>
                            <p className={`font-semibold ${totalPnl >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                                {totalPnl >= 0 ? '+' : ''}{totalPnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                                <span className="text-xs ml-1">
                                    ({totalPnlPct! >= 0 ? '+' : ''}{totalPnlPct!.toFixed(2)}%)
                                </span>
                            </p>
                        </div>
                    </div>
                )}

                {/* Table */}
                <div className="overflow-auto flex-1">
                    {loading ? (
                        <p className="text-center py-8 text-neutral-500">{t('loading')}</p>
                    ) : error ? (
                        <p className="text-center py-8 text-red-500">{t('errorLoad')}</p>
                    ) : records.length === 0 ? (
                        <p className="text-center py-8 text-neutral-500">{t('noHistory')}</p>
                    ) : (
                        <table className="w-full min-w-[480px] text-sm">
                            <thead className="sticky top-0 bg-white dark:bg-neutral-800 border-b dark:border-neutral-700">
                                <tr>
                                    <th className="text-left px-4 py-2 font-medium text-neutral-500 dark:text-neutral-400">{t('date')}</th>
                                    <th className="text-right px-4 py-2 font-medium text-neutral-500 dark:text-neutral-400">{t('closePrice')}</th>
                                    <th className="text-right px-4 py-2 font-medium text-neutral-500 dark:text-neutral-400">{t('dailyChange')}</th>
                                    <th className="text-right px-4 py-2 font-medium text-neutral-500 dark:text-neutral-400">{t('pnl')}</th>
                                    <th className="text-right px-4 py-2 font-medium text-neutral-500 dark:text-neutral-400">{t('pnlPercent')}</th>
                                </tr>
                            </thead>
                            <tbody>
                                {records.map((r, i) => {
                                    const chg = dailyChange(i);
                                    return (
                                        <tr key={r.date} className="border-b dark:border-neutral-700 hover:bg-neutral-50 dark:hover:bg-neutral-700/50">
                                            <td className="px-4 py-2">{r.date}</td>
                                            <td className="px-4 py-2 text-right">{r.close.toFixed(2)}</td>
                                            <td className={`px-4 py-2 text-right ${chg > 0 ? 'text-red-500' : chg < 0 ? 'text-green-500' : ''}`}>
                                                {chg > 0 ? '+' : ''}{chg.toFixed(2)}
                                            </td>
                                            <td className={`px-4 py-2 text-right font-medium ${r.pnl >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                                                {r.pnl >= 0 ? '+' : ''}{r.pnl.toLocaleString(undefined, { maximumFractionDigits: 0 })}
                                            </td>
                                            <td className={`px-4 py-2 text-right ${r.pnl_pct >= 0 ? 'text-red-500' : 'text-green-500'}`}>
                                                {r.pnl_pct >= 0 ? '+' : ''}{r.pnl_pct.toFixed(2)}%
                                            </td>
                                        </tr>
                                    );
                                })}
                            </tbody>
                        </table>
                    )}
                </div>
            </div>
        </div>
    );
}
