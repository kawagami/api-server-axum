"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslations, useLocale } from "next-intl";
import { Link } from "@/i18n/navigation";
import { Trash2, Loader2, QrCode, ScanBarcode, Keyboard, Trophy, Plus, X } from "lucide-react";
import { getInvoices, deleteInvoice } from "@/api/invoices";
import type { Invoice, InvoiceSource, PrizeTier } from "@/types";

const PER_PAGE = 50;

interface Props {
    initialEntries: Invoice[];
    lockWon?: boolean; // 我的中獎頁鎖 won=true、隱藏 filter
}

type WonFilter = '' | 'won' | 'notWon';

const SOURCE_ICON: Record<InvoiceSource, typeof QrCode> = {
    qr: QrCode,
    barcode: ScanBarcode,
    manual: Keyboard,
};

const PRIZE_KEY: Record<PrizeTier, string> = {
    special: 'prizeSpecial',
    grand: 'prizeGrand',
    first: 'prizeFirst',
    second: 'prizeSecond',
    third: 'prizeThird',
    fourth: 'prizeFourth',
    fifth: 'prizeFifth',
    sixth: 'prizeSixth',
    additional_sixth: 'prizeAdditionalSixth',
};

function fmtAmount(s: string | null) {
    if (s === null) return '—';
    return Number(s).toLocaleString(undefined, { maximumFractionDigits: 2 });
}

export default function InvoiceListClient({ initialEntries, lockWon = false }: Props) {
    const t = useTranslations('Invoices');
    const locale = useLocale();
    const dateFmt = new Intl.DateTimeFormat(locale, { year: 'numeric', month: '2-digit', day: '2-digit', timeZone: 'Asia/Taipei' });

    const [entries, setEntries] = useState<Invoice[]>(initialEntries);
    const [period, setPeriod] = useState('');
    const [won, setWon] = useState<WonFilter>('');
    const [page, setPage] = useState(1);
    const [hasMore, setHasMore] = useState(initialEntries.length >= PER_PAGE);
    const [loading, setLoading] = useState(false);
    const [loadingMore, setLoadingMore] = useState(false);
    const [mutating, setMutating] = useState(false);
    const firstRun = useRef(true);

    const wonParam = useCallback((): boolean | undefined => {
        if (lockWon) return true;
        if (won === 'won') return true;
        if (won === 'notWon') return false;
        return undefined;
    }, [lockWon, won]);

    const reload = useCallback(async () => {
        const list = await getInvoices({ period: period || undefined, won: wonParam(), page: 1, per_page: PER_PAGE });
        setEntries(list);
        setPage(1);
        setHasMore(list.length >= PER_PAGE);
    }, [period, wonParam]);

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
            const more = await getInvoices({ period: period || undefined, won: wonParam(), page: next, per_page: PER_PAGE });
            setEntries(prev => [...prev, ...more]);
            setPage(next);
            setHasMore(more.length >= PER_PAGE);
        } finally {
            setLoadingMore(false);
        }
    }

    async function handleDelete(id: string) {
        if (!confirm(t('confirmDelete'))) return;
        setMutating(true);
        try {
            await deleteInvoice(id);
            setEntries(prev => prev.filter(e => e.id !== id));
        } finally {
            setMutating(false);
        }
    }

    function statusBadge(inv: Invoice) {
        if (!inv.lottery_checked) {
            return <span className="px-2 py-0.5 text-xs rounded-full bg-neutral-100 dark:bg-neutral-700 text-neutral-500 dark:text-neutral-400">{t('statusPending')}</span>;
        }
        if (inv.prize_tier === null) {
            return <span className="px-2 py-0.5 text-xs rounded-full bg-neutral-100 dark:bg-neutral-700 text-neutral-400 dark:text-neutral-500">{t('statusNotWon')}</span>;
        }
        return (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300 font-medium">
                <Trophy size={12} />
                {t(PRIZE_KEY[inv.prize_tier])}
            </span>
        );
    }

    const filterActive = period || won;

    return (
        <div className="flex flex-col gap-4">
            {!lockWon && (
                <>
                    <div className="flex justify-end gap-2">
                        <Link
                            href="/invoices/scan"
                            className="flex items-center gap-2 px-4 py-2 rounded bg-primary-500 text-white hover:bg-primary-600 text-sm"
                        >
                            <Plus size={16} />
                            {t('register')}
                        </Link>
                    </div>

                    {/* 篩選列 */}
                    <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow border dark:border-neutral-700 flex flex-wrap items-end gap-3">
                        <div className="flex flex-col gap-1">
                            <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('period')}</label>
                            <input
                                type="text"
                                value={period}
                                onChange={e => setPeriod(e.target.value.replace(/\D/g, '').slice(0, 6))}
                                placeholder="202606"
                                inputMode="numeric"
                                className="border rounded px-2 py-1.5 text-sm w-28 font-mono dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                            />
                        </div>
                        <div className="flex flex-col gap-1">
                            <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('lotteryStatus')}</label>
                            <select
                                value={won}
                                onChange={e => setWon(e.target.value as WonFilter)}
                                className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                            >
                                <option value="">{t('all')}</option>
                                <option value="won">{t('filterWon')}</option>
                                <option value="notWon">{t('filterNotWon')}</option>
                            </select>
                        </div>
                        {filterActive && (
                            <button
                                onClick={() => { setPeriod(''); setWon(''); }}
                                className="flex items-center gap-1 px-3 py-1.5 text-sm rounded border dark:border-neutral-600 hover:bg-neutral-100 dark:hover:bg-neutral-700"
                            >
                                <X size={14} />
                                {t('clearFilters')}
                            </button>
                        )}
                    </div>
                </>
            )}

            <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('disclaimer')}</p>

            {loading ? (
                <div className="flex justify-center py-12 text-neutral-400">
                    <Loader2 className="animate-spin" size={24} />
                </div>
            ) : entries.length === 0 ? (
                <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{lockWon ? t('noWinnings') : t('noInvoices')}</p>
            ) : (
                <div className="flex flex-col gap-2">
                    {entries.map(inv => {
                        const SourceIcon = SOURCE_ICON[inv.source];
                        return (
                            <div
                                key={inv.id}
                                className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow border dark:border-neutral-700 flex items-center gap-3"
                            >
                                <SourceIcon size={16} className="text-neutral-400 shrink-0" aria-label={t(`source_${inv.source}`)} />
                                <div className="flex flex-col gap-0.5 min-w-0 flex-1">
                                    <div className="flex items-center gap-2 flex-wrap">
                                        <span className="font-mono font-medium">{inv.invoice_number}</span>
                                        {statusBadge(inv)}
                                    </div>
                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                        {dateFmt.format(new Date(inv.invoice_date))}
                                        {inv.amount !== null && <> · {fmtAmount(inv.amount)}</>}
                                    </span>
                                </div>
                                <button
                                    onClick={() => handleDelete(inv.id)}
                                    disabled={mutating}
                                    title={t('delete')}
                                    className="p-2 rounded hover:bg-neutral-100 dark:hover:bg-neutral-700 text-red-500 disabled:opacity-50 shrink-0"
                                >
                                    <Trash2 size={16} />
                                </button>
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
        </div>
    );
}
