"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { useTranslations, useLocale } from "next-intl";
import { Loader2 } from "lucide-react";
import { getLottoDraws } from "@/api/lotto";
import { GAME_KEY, LOTTO_GAMES } from "@/libs/lotto";
import Balls from "@/components/lotto/Balls";
import type { LottoDraw, LottoGame } from "@/types";

const LIMIT = 20;

export default function LottoDrawsClient({ initialDraws }: { initialDraws: LottoDraw[] }) {
    const t = useTranslations('Lotto');
    const locale = useLocale();
    const dateFmt = new Intl.DateTimeFormat(locale, { year: 'numeric', month: '2-digit', day: '2-digit', timeZone: 'Asia/Taipei' });

    const [draws, setDraws] = useState<LottoDraw[]>(initialDraws);
    const [game, setGame] = useState<'' | LottoGame>('');
    const [loading, setLoading] = useState(false);
    const firstRun = useRef(true);

    const reload = useCallback(async () => {
        const list = await getLottoDraws({ game: game || undefined, limit: LIMIT });
        setDraws(list);
    }, [game]);

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

    return (
        <div className="flex flex-col gap-4">
            <div className="flex flex-wrap gap-2">
                {([['', 'all'], ...LOTTO_GAMES.map(g => [g, GAME_KEY[g]] as const)] as const).map(([value, key]) => (
                    <button
                        key={value || 'all'}
                        onClick={() => setGame(value as '' | LottoGame)}
                        className={`px-3 py-1.5 rounded-full text-sm transition-colors ${game === value
                            ? 'bg-primary-500 text-white'
                            : 'border border-neutral-200 dark:border-neutral-700 hover:bg-neutral-100 dark:hover:bg-neutral-800'}`}
                    >
                        {t(key)}
                    </button>
                ))}
            </div>

            <p className="text-xs text-neutral-400 dark:text-neutral-500">{t('drawsHint')}</p>

            {loading ? (
                <div className="flex justify-center py-12 text-neutral-400">
                    <Loader2 className="animate-spin" size={24} />
                </div>
            ) : draws.length === 0 ? (
                <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{t('noDraws')}</p>
            ) : (
                <div className="flex flex-col gap-2">
                    {draws.map(d => (
                        <div
                            key={`${d.game}-${d.period}`}
                            className="bg-white dark:bg-neutral-800 rounded-xl px-4 py-3 shadow border dark:border-neutral-700 flex flex-col gap-1.5"
                        >
                            <div className="flex items-center gap-2 flex-wrap text-sm">
                                <span className="font-medium">{t(GAME_KEY[d.game])}</span>
                                <span className="text-xs text-neutral-500 dark:text-neutral-400">{dateFmt.format(new Date(d.draw_date))}</span>
                                <span className="text-xs text-neutral-400 dark:text-neutral-500 font-mono">#{d.period}</span>
                            </div>
                            <Balls main={d.main_nums} special={d.special} />
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
