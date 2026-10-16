"use client";

import { useTranslations } from 'next-intl';
import { Speech, Volume2, VolumeX, RotateCcw } from 'lucide-react';
import { cn } from '@/libs/cn';
import { HARVEST_ROUNDS, TOTAL_ROUNDS, type FarmState, type PlayerState } from './farm-types';
import { feedingOutlook, nextHarvest } from './farm-rules';
import { seatColor } from './farm-ui';

const ICON_BTN = 'flex items-center gap-1.5 rounded-lg border border-neutral-300 p-2 text-sm text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800';

/// 頂部「現在狀況」橫幅：輪到誰、輪數進度、收穫糧食預估、語音字幕 + 再聽一次、錯誤提示。
export function FarmStatus({
    state, mySeat, seatName, narration, notice, voice, muted, onToggleMute,
}: {
    state: FarmState;
    mySeat: number | null;
    seatName: (s: number) => string;
    narration: string;
    notice: string | null;
    voice: { supported: boolean; enabled: boolean; toggle: () => void; replay: (text: string) => void };
    muted: boolean;
    onToggleMute: () => void;
}) {
    const t = useTranslations('Farm');
    const tl = useTranslations('GameLobby');
    const cur = state.current_player;
    const myTurn = state.phase === 'placing' && cur !== null && cur === mySeat;
    const me = mySeat !== null ? state.players[mySeat] : undefined;

    return (
        <section className={cn(
            'sticky top-12.5 z-20 flex flex-col gap-2 rounded-lg border bg-white/90 p-3 shadow-sm backdrop-blur-xs dark:bg-neutral-900/90',
            myTurn ? 'border-primary-500 ring-2 ring-primary-400/60' : 'border-neutral-200 dark:border-neutral-700',
        )}>
            <div className="flex flex-wrap items-center justify-between gap-2">
                <p className="flex items-center gap-2 text-lg font-bold text-neutral-800 dark:text-neutral-100">
                    {cur !== null && <span className={cn('h-3 w-3 flex-none rounded-full', seatColor(cur).dot)} />}
                    {state.phase === 'game_over' || cur === null ? t('settling')
                        : myTurn ? <span className="text-primary-600 dark:text-primary-400">{t('yourTurnWorkers', { n: state.workers_left[cur] ?? 0 })}</span>
                            : t('currentTurn', { name: seatName(cur) })}
                </p>
                <div className="flex items-center gap-1.5">
                    {voice.supported && (<>
                        <button type="button" onClick={() => voice.replay(narration)} className={ICON_BTN}>
                            <RotateCcw className="h-4 w-4" /><span>{t('replay')}</span>
                        </button>
                        <button type="button" onClick={voice.toggle} aria-pressed={voice.enabled}
                            aria-label={voice.enabled ? t('voiceOff') : t('voiceOn')} title={voice.enabled ? t('voiceOff') : t('voiceOn')}
                            className={cn(ICON_BTN, voice.enabled ? 'text-primary-600 dark:text-primary-400' : 'opacity-60')}>
                            <Speech className="h-5 w-5" />
                        </button>
                    </>)}
                    <button type="button" onClick={onToggleMute} aria-label={muted ? tl('soundOn') : tl('soundOff')} className={ICON_BTN}>
                        {muted ? <VolumeX className="h-5 w-5" /> : <Volume2 className="h-5 w-5" />}
                    </button>
                </div>
            </div>

            <RoundTrack round={state.round} />

            {me && state.phase === 'placing' && <FeedLine round={state.round} me={me} />}

            {narration && (
                <p className="rounded-md bg-neutral-100 px-2.5 py-1.5 text-sm text-neutral-700 dark:bg-neutral-800 dark:text-neutral-200" aria-live="polite">
                    📢 {narration}
                </p>
            )}

            {notice && (
                <p role="alert" className="rounded-md border border-red-300 bg-red-50 px-2.5 py-1.5 text-sm text-red-600 dark:border-red-800 dark:bg-red-950 dark:text-red-400">
                    {t(notice)}
                </p>
            )}
        </section>
    );
}

/** 14 輪進度條：已過 / 本輪 / 未來，收穫輪標 🌾 */
function RoundTrack({ round }: { round: number }) {
    const t = useTranslations('Farm');
    return (
        <div className="flex flex-col gap-1">
            <div className="flex items-center justify-between text-xs text-neutral-500 dark:text-neutral-400">
                <span>{t('round', { n: round })}</span>
                {HARVEST_ROUNDS.has(round) && <span className="rounded-full bg-amber-100 px-2 py-0.5 font-medium text-amber-700 dark:bg-amber-900 dark:text-amber-200">🌾 {t('harvestRound')}</span>}
            </div>
            <ol className="flex gap-0.5" aria-hidden="true">
                {Array.from({ length: TOTAL_ROUNDS }, (_, i) => i + 1).map(r => (
                    <li key={r} className="flex flex-1 flex-col items-center gap-0.5">
                        <span className={cn('h-2 w-full rounded-sm',
                            r < round ? 'bg-primary-400 dark:bg-primary-600'
                                : r === round ? 'bg-primary-600 ring-2 ring-primary-300 dark:bg-primary-400'
                                    : HARVEST_ROUNDS.has(r) ? 'bg-amber-300 dark:bg-amber-700'
                                        : 'bg-neutral-200 dark:bg-neutral-700')} />
                        <span className="h-3 text-[10px] leading-3">{HARVEST_ROUNDS.has(r) ? '🌾' : ''}</span>
                    </li>
                ))}
            </ol>
        </div>
    );
}

/** 下次收穫要餵多少糧、夠不夠（夠 / 要煮食 / 會乞討 三段色） */
function FeedLine({ round, me }: { round: number; me: PlayerState }) {
    const t = useTranslations('Farm');
    const h = nextHarvest(round);
    if (h === null) return null;
    const f = feedingOutlook(me);
    const level = f.shortfall > 0 ? 'short' : f.food < f.need ? 'convert' : 'ok';
    return (
        <p className={cn('flex flex-wrap items-center gap-x-2 text-sm',
            level === 'short' ? 'font-semibold text-red-600 dark:text-red-400'
                : level === 'convert' ? 'text-amber-700 dark:text-amber-300'
                    : 'text-neutral-600 dark:text-neutral-300')}>
            <span>🍲 {h === round ? t('harvestThisRound') : t('harvestIn', { round: h, n: h - round })}</span>
            <span>{t(`feed_${level}`, { need: f.need, food: f.food, n: f.shortfall })}</span>
        </p>
    );
}
