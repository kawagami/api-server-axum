"use client";

import { useEffect, useState } from "react";
import { useTranslations } from "next-intl";
import { getPortfolioHistory } from "@/api/portfolio";
import type { HistoryRecord, PortfolioSummaryEntry, PeriodKey } from "@/types";
import Modal from "@/components/modal";
import PeriodTabs from "./period-tabs";

/** 「今日」在逐日表格裡只有一列，所以這裡的選項是近一週 / 近一月 / 全部。 */
const RANGES = ['week', 'month', 'all'] as const;
type Range = typeof RANGES[number];

interface Props {
    entry: PortfolioSummaryEntry;
    /** 總覽選的期間；`day` 沒有對應的表格區間，退回「全部」 */
    period: PeriodKey;
    onClose: () => void;
}

export default function StockHistoryTable({ entry, period, onClose }: Props) {
    const t = useTranslations('Portfolio');
    const [records, setRecords] = useState<HistoryRecord[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState(false);
    const [range, setRange] = useState<Range>(period === 'day' ? 'all' : period);

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

    // 區間起點直接用後端算增減時的基準日 —— 自己在前端重推一次日期，就會有
    // 「表格從哪天開始」與「卡片上那個 % 從哪天算起」對不起來的機會（JS 的
    // setUTCMonth 跨月會溢位，chrono 的 checked_sub_months 是夾到當月最後一天）。
    // null = 該期間沒有基準日（持股太新／行情有洞），那本來就沒有更早的資料可濾。
    const from = range === 'all' ? null : (entry.changes[range]?.base_date ?? null);

    // 逐日漲跌（後端不回，本地算）。**先算完整序列再濾區間** —— 反過來的話
    // 區間第一列會顯示 0，而它其實相對前一交易日有漲跌。
    const rows = records
        .map((r, i) => ({ r, chg: i === 0 ? 0 : r.close - records[i - 1].close }))
        .filter(({ r }) => from === null || r.date >= from);

    return (
        <Modal
            label={`${entry.stock_code} ${t('historyTitle')}`}
            onClose={onClose}
            size="xl"
            surface="public"
            className="max-h-[85vh] flex flex-col"
        >
            {/* Header */}
            <div className="flex items-center justify-between px-5 py-4 border-b dark:border-neutral-700">
                <div>
                    <h2 className="font-bold text-lg">{entry.stock_code} {t('historyTitle')}</h2>
                    <p className="text-xs text-neutral-500 dark:text-neutral-400">
                        {t('buyDate')}: {entry.buy_date} · {t('costPerShare')}: {entry.cost_per_share} · {t('shares')}: {entry.shares}
                    </p>
                </div>
                <div className="flex items-center gap-3">
                    <PeriodTabs options={RANGES} value={range} onChange={setRange} label={r => t(`period.${r}`)} />
                    <button
                        onClick={onClose}
                        className="text-neutral-500 hover:text-neutral-800 dark:hover:text-neutral-200 text-xl leading-none px-2"
                    >
                        ✕
                    </button>
                </div>
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
                ) : rows.length === 0 ? (
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
                            {rows.map(({ r, chg }) => ((
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
                            )))}
                        </tbody>
                    </table>
                )}
            </div>
        </Modal>
    );
}
