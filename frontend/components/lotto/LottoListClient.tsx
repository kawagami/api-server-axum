"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslations, useLocale } from "next-intl";
import { Link } from "@/i18n/navigation";
import { Trash2, Loader2, QrCode, Keyboard, Trophy, Plus, X } from "lucide-react";
import { getLottoTickets, deleteLottoTicket } from "@/api/lotto";
import { GAME_KEY, PRIZE_KEY } from "@/libs/lotto";
import Balls from "@/components/lotto/Balls";
import type { LottoTicket, LottoSource, LottoGame, LottoStatus } from "@/types";

const PER_PAGE = 50;

interface Props {
    initialEntries: LottoTicket[];
    lockWon?: boolean; // 我的中獎頁鎖 status=won、隱藏 filter
}

const SOURCE_ICON: Record<LottoSource, typeof QrCode> = {
    qr: QrCode,
    manual: Keyboard,
};

export default function LottoListClient({ initialEntries, lockWon = false }: Props) {
    const t = useTranslations('Lotto');
    const locale = useLocale();
    const dateFmt = new Intl.DateTimeFormat(locale, { year: 'numeric', month: '2-digit', day: '2-digit', timeZone: 'Asia/Taipei' });

    const [entries, setEntries] = useState<LottoTicket[]>(initialEntries);
    const [game, setGame] = useState<'' | LottoGame>('');
    const [status, setStatus] = useState<'' | LottoStatus>('');
    const [page, setPage] = useState(1);
    const [hasMore, setHasMore] = useState(initialEntries.length >= PER_PAGE);
    const [loading, setLoading] = useState(false);
    const [loadingMore, setLoadingMore] = useState(false);
    const [mutating, setMutating] = useState(false);
    const firstRun = useRef(true);

    const statusParam = useCallback((): LottoStatus | undefined => {
        if (lockWon) return 'won';
        return status || undefined;
    }, [lockWon, status]);

    const reload = useCallback(async () => {
        const list = await getLottoTickets({ game: game || undefined, status: statusParam(), page: 1, per_page: PER_PAGE });
        setEntries(list);
        setPage(1);
        setHasMore(list.length >= PER_PAGE);
    }, [game, statusParam]);

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
            const more = await getLottoTickets({ game: game || undefined, status: statusParam(), page: next, per_page: PER_PAGE });
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
            await deleteLottoTicket(id);
            setEntries(prev => prev.filter(e => e.id !== id));
        } finally {
            setMutating(false);
        }
    }

    function statusBadge(ticket: LottoTicket) {
        if (!ticket.checked) {
            return <span className="px-2 py-0.5 text-xs rounded-full bg-neutral-100 dark:bg-neutral-700 text-neutral-500 dark:text-neutral-400">{t('statusPending')}</span>;
        }
        if (ticket.prize_tier === null) {
            return <span className="px-2 py-0.5 text-xs rounded-full bg-neutral-100 dark:bg-neutral-700 text-neutral-400 dark:text-neutral-500">{t('statusNotWon')}</span>;
        }
        return (
            <span className="inline-flex items-center gap-1 px-2 py-0.5 text-xs rounded-full bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300 font-medium">
                <Trophy size={12} />
                {t('won', { prize: t(PRIZE_KEY[ticket.prize_tier]) })}
            </span>
        );
    }

    const filterActive = game || status;

    return (
        <div className="flex flex-col gap-4">
            {!lockWon && (
                <>
                    <div className="flex justify-end">
                        <Link
                            href="/lotto/register"
                            className="flex items-center gap-2 px-4 py-2 rounded bg-primary-500 text-white hover:bg-primary-600 text-sm"
                        >
                            <Plus size={16} />
                            {t('navRegister')}
                        </Link>
                    </div>

                    {/* 篩選列 */}
                    <div className="bg-white dark:bg-neutral-800 rounded-xl p-4 shadow border dark:border-neutral-700 flex flex-wrap items-end gap-3">
                        <div className="flex flex-col gap-1">
                            <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('game')}</label>
                            <select
                                value={game}
                                onChange={e => setGame(e.target.value as '' | LottoGame)}
                                className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                            >
                                <option value="">{t('all')}</option>
                                <option value="lotto649">{t('gameLotto649')}</option>
                                <option value="super_lotto638">{t('gameSuperLotto638')}</option>
                            </select>
                        </div>
                        <div className="flex flex-col gap-1">
                            <label className="text-xs text-neutral-500 dark:text-neutral-400">{t('status')}</label>
                            <select
                                value={status}
                                onChange={e => setStatus(e.target.value as '' | LottoStatus)}
                                className="border rounded px-2 py-1.5 text-sm dark:bg-neutral-700 dark:border-neutral-600 focus:outline-none focus:ring-2 focus:ring-primary-400"
                            >
                                <option value="">{t('all')}</option>
                                <option value="pending">{t('filterPending')}</option>
                                <option value="won">{t('filterWon')}</option>
                                <option value="lost">{t('filterLost')}</option>
                            </select>
                        </div>
                        {filterActive && (
                            <button
                                onClick={() => { setGame(''); setStatus(''); }}
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
                <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{lockWon ? t('noWinnings') : t('noTickets')}</p>
            ) : (
                <div className="flex flex-col gap-2">
                    {entries.map(ticket => {
                        const SourceIcon = SOURCE_ICON[ticket.source];
                        return (
                            <div
                                key={ticket.id}
                                className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow border dark:border-neutral-700 flex items-center gap-3"
                            >
                                <div className="flex flex-col gap-1.5 min-w-0 flex-1">
                                    <div className="flex items-center gap-2 flex-wrap">
                                        <span className="text-sm font-medium">{t(GAME_KEY[ticket.game])}</span>
                                        {statusBadge(ticket)}
                                        <SourceIcon size={13} className="text-neutral-400" aria-label={t(`source_${ticket.source}`)} />
                                    </div>
                                    <Balls main={ticket.picks} special={ticket.second} size="sm" />
                                    <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                        {t('drawDate')}: {dateFmt.format(new Date(ticket.draw_date))}
                                    </span>
                                </div>
                                <button
                                    onClick={() => handleDelete(ticket.id)}
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
