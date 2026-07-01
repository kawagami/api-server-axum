"use client";

import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { useTranslations } from "next-intl";
import { Plus, Pencil, Trash2, Loader2, X, ScanLine, Receipt } from "lucide-react";
import { getLedger, getLedgerSummary, postLedger, putLedger, deleteLedger } from "@/api/ledger";
import LedgerForm from "@/components/ledger/LedgerForm";
import InvoiceImportModal from "@/components/ledger/InvoiceImportModal";
import { CategoryPie, MonthlyBars } from "@/components/ledger/ledger-charts";
import type { LedgerEntry, LedgerInput, LedgerSummary, LedgerCategories, LedgerKind } from "@/types";

const PER_PAGE = 50;

interface Props {
    categories: LedgerCategories;
    initialEntries: LedgerEntry[];
    initialSummary: LedgerSummary;
}

interface Filters {
    kind: '' | LedgerKind;
    category: string;
    from: string;
    to: string;
}

type Mode =
    | { type: 'list' }
    | { type: 'add' }
    | { type: 'edit'; entry: LedgerEntry }
    | { type: 'scan' };

function fmt(s: string) {
    return Number(s).toLocaleString(undefined, { maximumFractionDigits: 2 });
}

function SummaryCard({ label, value, tone }: { label: string; value: string; tone?: 'income' | 'expense' | 'balance'; }) {
    const color =
        tone === 'income' ? 'text-green-600 dark:text-green-400'
            : tone === 'expense' ? 'text-red-500 dark:text-red-400'
                : tone === 'balance' ? (Number(value.replace(/[^\d.-]/g, '')) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-500 dark:text-red-400')
                    : '';
    return (
        <div className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow border dark:border-neutral-700">
            <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-1">{label}</p>
            <p className={`font-semibold text-lg tabular-nums ${color}`}>{value}</p>
        </div>
    );
}

export default function LedgerClient({ categories, initialEntries, initialSummary }: Props) {
    const t = useTranslations('Ledger');
    const [entries, setEntries] = useState<LedgerEntry[]>(initialEntries);
    const [summary, setSummary] = useState<LedgerSummary>(initialSummary);
    const [filters, setFilters] = useState<Filters>({ kind: '', category: '', from: '', to: '' });
    const [page, setPage] = useState(1);
    const [hasMore, setHasMore] = useState(initialEntries.length >= PER_PAGE);
    const [mode, setMode] = useState<Mode>({ type: 'list' });
    const [loading, setLoading] = useState(false);
    const [loadingMore, setLoadingMore] = useState(false);
    const [mutating, setMutating] = useState(false);
    const firstRun = useRef(true);

    // value→label 查表（清單與圓餅圖共用）
    const labelOf = useCallback((kind: LedgerKind, value: string) => {
        return categories[kind]?.find(o => o.value === value)?.label ?? value;
    }, [categories]);

    const reload = useCallback(async () => {
        const [list, sum] = await Promise.all([
            getLedger({ ...filters, page: 1, per_page: PER_PAGE }),
            getLedgerSummary({ from: filters.from, to: filters.to }),
        ]);
        setEntries(list);
        setSummary(sum);
        setPage(1);
        setHasMore(list.length >= PER_PAGE);
    }, [filters]);

    // filter 變動 → 重抓清單與統計（首次 render 由 server 端資料 seed，跳過）
    useEffect(() => {
        if (firstRun.current) {
            firstRun.current = false;
            return;
        }
        let cancelled = false;
        setLoading(true);
        reload()
            .catch(() => { /* memberRequest 處理 401 redirect */ })
            .finally(() => { if (!cancelled) setLoading(false); });
        return () => { cancelled = true; };
    }, [reload]);

    async function loadMore() {
        setLoadingMore(true);
        try {
            const next = page + 1;
            const more = await getLedger({ ...filters, page: next, per_page: PER_PAGE });
            setEntries(prev => [...prev, ...more]);
            setPage(next);
            setHasMore(more.length >= PER_PAGE);
        } finally {
            setLoadingMore(false);
        }
    }

    async function handleSave(input: LedgerInput, id?: string) {
        if (id) await putLedger(id, input);
        else await postLedger(input);
        setMode({ type: 'list' });
        setLoading(true);
        try {
            await reload();
        } finally {
            setLoading(false);
        }
    }

    async function handleImported() {
        setMode({ type: 'list' });
        setLoading(true);
        try {
            await reload();
        } finally {
            setLoading(false);
        }
    }

    async function handleDelete(id: string) {
        if (!confirm(t('confirmDelete'))) return;
        setMutating(true);
        try {
            await deleteLedger(id);
            await reload();
        } finally {
            setMutating(false);
        }
    }

    const filterActive = filters.kind || filters.category || filters.from || filters.to;
    const filterCategoryOptions = filters.kind ? categories[filters.kind] : [];

    // by_category 拆成支出 / 收入兩組圓餅
    const expenseSlices = useMemo(
        () => summary.by_category.filter(c => c.kind === 'expense').map(c => ({ label: labelOf('expense', c.category), value: Number(c.total) })),
        [summary.by_category, labelOf]
    );
    const incomeSlices = useMemo(
        () => summary.by_category.filter(c => c.kind === 'income').map(c => ({ label: labelOf('income', c.category), value: Number(c.total) })),
        [summary.by_category, labelOf]
    );

    return (
        <div className="flex flex-col gap-4">
            {/* 統計總覽 */}
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <SummaryCard label={t('totalIncome')} value={fmt(summary.total_income)} tone="income" />
                <SummaryCard label={t('totalExpense')} value={fmt(summary.total_expense)} tone="expense" />
                <SummaryCard label={t('balance')} value={fmt(summary.balance)} tone="balance" />
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <CategoryPie title={t('expenseByCategory')} slices={expenseSlices} />
                <CategoryPie title={t('incomeByCategory')} slices={incomeSlices} />
            </div>
            <MonthlyBars monthly={summary.monthly} />

            {/* 篩選列 */}
            <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow border dark:border-neutral-700 flex flex-wrap items-end gap-3">
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('kind')}</label>
                    <select
                        value={filters.kind}
                        onChange={e => setFilters(f => ({ ...f, kind: e.target.value as Filters['kind'], category: '' }))}
                        className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                    >
                        <option value="">{t('all')}</option>
                        <option value="income">{t('income')}</option>
                        <option value="expense">{t('expense')}</option>
                    </select>
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('category')}</label>
                    <select
                        value={filters.category}
                        onChange={e => setFilters(f => ({ ...f, category: e.target.value }))}
                        disabled={!filters.kind}
                        className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400 disabled:opacity-50"
                    >
                        <option value="">{t('all')}</option>
                        {filterCategoryOptions.map(o => (
                            <option key={o.value} value={o.value}>{o.label}</option>
                        ))}
                    </select>
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('from')}</label>
                    <input
                        type="date"
                        value={filters.from}
                        onChange={e => setFilters(f => ({ ...f, from: e.target.value }))}
                        className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('to')}</label>
                    <input
                        type="date"
                        value={filters.to}
                        onChange={e => setFilters(f => ({ ...f, to: e.target.value }))}
                        className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                    />
                </div>
                {filterActive && (
                    <button
                        onClick={() => setFilters({ kind: '', category: '', from: '', to: '' })}
                        className="flex items-center gap-1 px-3 py-1.5 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                    >
                        <X size={14} />
                        {t('clearFilters')}
                    </button>
                )}
            </div>

            {/* 新增按鈕 / 表單 */}
            {mode.type === 'list' && (
                <div className="flex justify-end gap-2">
                    <button
                        onClick={() => setMode({ type: 'scan' })}
                        className="flex items-center gap-2 px-4 py-2 rounded border border-primary-500 text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-950 text-sm"
                    >
                        <ScanLine size={16} />
                        {t('scanInvoice')}
                    </button>
                    <button
                        onClick={() => setMode({ type: 'add' })}
                        className="flex items-center gap-2 px-4 py-2 rounded bg-primary-500 text-white hover:bg-primary-600 text-sm"
                    >
                        <Plus size={16} />
                        {t('addEntry')}
                    </button>
                </div>
            )}
            {mode.type === 'add' && (
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
                    <h2 className="font-semibold mb-4">{t('addEntry')}</h2>
                    <LedgerForm categories={categories} onSave={input => handleSave(input)} onCancel={() => setMode({ type: 'list' })} />
                </div>
            )}
            {mode.type === 'edit' && (
                <div className="bg-white dark:bg-neutral-800 rounded-xl p-5 shadow border dark:border-neutral-700">
                    <h2 className="font-semibold mb-4">{t('editEntry')}</h2>
                    <LedgerForm categories={categories} initial={mode.entry} onSave={input => handleSave(input, mode.entry.id)} onCancel={() => setMode({ type: 'list' })} />
                </div>
            )}

            {/* 清單 */}
            {loading ? (
                <div className="flex justify-center py-12 text-neutral-400">
                    <Loader2 className="animate-spin" size={24} />
                </div>
            ) : entries.length === 0 ? (
                <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{t('noEntries')}</p>
            ) : (
                <div className="flex flex-col gap-2">
                    {entries.map(entry => {
                        const income = entry.kind === 'income';
                        return (
                            <div
                                key={entry.id}
                                className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow border dark:border-neutral-700 flex items-center gap-3"
                            >
                                <span className={`w-1 self-stretch rounded-full ${income ? 'bg-green-500' : 'bg-red-500'}`} aria-hidden="true" />
                                <div className="flex flex-col gap-0.5 min-w-0 flex-1">
                                    <div className="flex items-baseline gap-2">
                                        <span className="font-medium">{labelOf(entry.kind, entry.category)}</span>
                                        {entry.source === 'invoice_qr' && (
                                            <Receipt size={13} className="text-primary-500 shrink-0 self-center" aria-label={t('fromInvoice')} />
                                        )}
                                        {entry.note && <span className="text-sm text-neutral-500 dark:text-neutral-400 truncate">{entry.note}</span>}
                                    </div>
                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">{entry.occurred_at}</span>
                                </div>
                                <span className={`font-semibold tabular-nums shrink-0 ${income ? 'text-green-600 dark:text-green-400' : 'text-red-500 dark:text-red-400'}`}>
                                    {income ? '+' : '-'}{fmt(entry.amount)}
                                </span>
                                <div className="flex items-center gap-1 shrink-0">
                                    <button
                                        onClick={() => setMode({ type: 'edit', entry })}
                                        title={t('editEntry')}
                                        className="p-2 rounded hover:bg-neutral-100 dark:hover:bg-neutral-700"
                                    >
                                        <Pencil size={16} />
                                    </button>
                                    <button
                                        onClick={() => handleDelete(entry.id)}
                                        disabled={mutating}
                                        title={t('deleteEntry')}
                                        className="p-2 rounded hover:bg-neutral-100 dark:hover:bg-neutral-700 text-red-500 disabled:opacity-50"
                                    >
                                        <Trash2 size={16} />
                                    </button>
                                </div>
                            </div>
                        );
                    })}

                    {hasMore && (
                        <button
                            onClick={loadMore}
                            disabled={loadingMore}
                            className="mt-2 self-center flex items-center gap-2 px-4 py-2 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700 disabled:opacity-50"
                        >
                            {loadingMore && <Loader2 className="animate-spin" size={14} />}
                            {t('loadMore')}
                        </button>
                    )}
                </div>
            )}

            {/* 掃發票匯入 */}
            {mode.type === 'scan' && (
                <InvoiceImportModal
                    categories={categories}
                    onClose={() => setMode({ type: 'list' })}
                    onImported={handleImported}
                />
            )}
        </div>
    );
}
