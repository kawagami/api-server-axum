"use client";

import { useTranslations } from 'next-intl';
import { cn } from '@/libs/cn';
import type { FarmEvent } from './useFarmRoom';
import { ACCUM_RES, ACTION_ICON, RES_ICON, seatColor } from './farm-ui';

/// 行動紀錄：新的在上。事件由 useFarmRoom 比對前後 state 推得。
export function FarmLog({ log, seatName }: { log: FarmEvent[]; seatName: (s: number) => string }) {
    const t = useTranslations('Farm');
    return (
        <section className="flex flex-col gap-2 rounded-lg border border-neutral-200 p-3 dark:border-neutral-700">
            <h2 className="text-sm font-semibold text-neutral-600 dark:text-neutral-300">{t('logHeading')}</h2>
            {log.length === 0 ? <p className="text-sm text-neutral-500">{t('logEmpty')}</p> : (
                <ol className="flex max-h-64 flex-col gap-1 overflow-y-auto text-sm">
                    {[...log].reverse().map((e, i) => (
                        <li key={`${e.seq}-${e.kind}-${i}`}>
                            {e.kind === 'round' ? (
                                <p className="my-1 text-center text-xs text-neutral-400">— {t('round', { n: e.round })} —</p>
                            ) : e.kind === 'harvest' ? (
                                <div className="rounded-md bg-amber-50 px-2 py-1 text-amber-800 dark:bg-amber-950 dark:text-amber-200">
                                    🌾 {t('logHarvest', { n: e.round })}
                                    {e.begging.map((b, seat) => b > 0 && (
                                        <span key={seat} className="ml-2 font-semibold text-red-600 dark:text-red-400">{t('logBegging', { name: seatName(seat), n: b })}</span>
                                    ))}
                                </div>
                            ) : (
                                <p className="flex items-center gap-1.5 text-neutral-700 dark:text-neutral-200">
                                    <span className={cn('h-2 w-2 flex-none rounded-full', seatColor(e.seat).dot)} />
                                    <span className={cn('font-medium', seatColor(e.seat).text)}>{seatName(e.seat)}</span>
                                    <span aria-hidden="true">{ACTION_ICON[e.action]}</span>
                                    <span>{t(`action_${e.action}`)}</span>
                                    {e.amount > 0 && ACCUM_RES[e.action] && (
                                        <span className="text-xs text-neutral-500">+{e.amount}{RES_ICON[ACCUM_RES[e.action]]}</span>
                                    )}
                                </p>
                            )}
                        </li>
                    ))}
                </ol>
            )}
        </section>
    );
}
