"use client";

import { useEffect, useEffectEvent } from 'react';
import { useTranslations } from 'next-intl';
import { RotateCcw } from 'lucide-react';
import { sound } from '../_shared/sound';
import { ROLE_ICON } from './AvalonRoleCard';
import { EVIL_ROLES, type GameOverData } from './avalon-types';

/// 結局遮罩（不可關閉、只能回大廳，所以不走 Modal）。出現時依自己陣營播勝利／失敗音效，角色逐列揭露
export function AvalonGameOver({ gameOver, isGood, mySeat, seatName, onBack }: {
    gameOver: GameOverData;
    isGood: boolean;
    mySeat: number;
    seatName: (seat: number) => string;
    onBack: () => void;
}) {
    const t = useTranslations('Avalon');
    const won = gameOver.winner !== null && (gameOver.winner === 'good') === isGood;

    const playEnd = useEffectEvent(() => {
        if (gameOver.winner === null) sound.gameOver();
        else if (won) sound.win();
        else sound.lose();
    });
    useEffect(() => { playEnd(); }, []);

    const banner = gameOver.winner === 'good' ? 'from-primary-600 to-primary-800'
        : gameOver.winner === 'evil' ? 'from-red-700 to-red-900' : 'from-neutral-600 to-neutral-800';

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-neutral-900/80 p-4 backdrop-blur-xs">
            <div role="dialog" aria-modal="true" aria-label={t('title')}
                className="flex max-h-full w-full max-w-md flex-col overflow-hidden rounded-xl bg-white shadow-2xl dark:bg-neutral-800">
                <div className={`animate-fade-in bg-linear-to-b ${banner} px-5 py-6 text-center text-white`}>
                    <h2 className="text-3xl font-bold">
                        {gameOver.winner === 'good' ? t('goodWins') : gameOver.winner === 'evil' ? t('evilWins') : t('draw')}
                    </h2>
                    <p className="mt-1 text-sm opacity-90">{t(`reason_${gameOver.reason}`)}</p>
                    {gameOver.winner !== null && (
                        <p className="mt-2 text-lg font-semibold">{won ? t('youWin') : t('youLose')}</p>
                    )}
                </div>
                <ul className="flex min-h-0 flex-col gap-1 overflow-y-auto p-4 text-sm">
                    {gameOver.roles.map((r, i) => {
                        const Icon = ROLE_ICON[r.role];
                        const evil = EVIL_ROLES.has(r.role);
                        return (
                            <li key={r.seat} style={{ animationDelay: `${150 + i * 90}ms`, animationFillMode: 'both' }}
                                className={`animate-fade-in flex items-center justify-between rounded-md px-2 py-1.5 odd:bg-neutral-100 dark:odd:bg-neutral-700 ${r.seat === mySeat ? 'font-bold' : ''}`}>
                                <span>{seatName(r.seat)}{r.seat === mySeat && `（${t('youTag')}）`}</span>
                                <span className={`flex items-center gap-1.5 ${evil ? 'text-red-600 dark:text-red-400' : 'text-primary-600 dark:text-primary-300'}`}>
                                    <Icon className="h-4 w-4" />{t(`role_${r.role}`)}
                                </span>
                            </li>
                        );
                    })}
                </ul>
                <div className="p-4 pt-0">
                    <button onClick={onBack} className="flex w-full items-center justify-center gap-2 rounded-lg bg-primary-600 px-4 py-2 font-medium text-white transition-colors hover:bg-primary-700">
                        <RotateCcw className="h-4 w-4" />{t('backToLobby')}
                    </button>
                </div>
            </div>
        </div>
    );
}
