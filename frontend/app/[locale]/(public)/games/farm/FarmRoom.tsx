"use client";

import { useTranslations } from 'next-intl';
import { Crown, Play, LogOut } from 'lucide-react';
import type { RoomUpdateData } from './farm-types';

export function FarmRoom({
    room, iAmHost, onStart, onLeave,
}: {
    room: RoomUpdateData;
    iAmHost: boolean;
    onStart: () => void;
    onLeave: () => void;
}) {
    const t = useTranslations('Farm');
    return (
        <div className="mx-auto flex w-full max-w-lg flex-col gap-4 py-6">
            <div className="flex items-center justify-between">
                <h1 className="truncate text-xl font-bold text-neutral-800 dark:text-neutral-100">{room.name}</h1>
                <button onClick={onLeave} className="flex flex-none items-center gap-1.5 rounded-lg border border-neutral-300 px-3 py-1.5 text-sm text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800">
                    <LogOut className="h-4 w-4" />{t('leave')}
                </button>
            </div>

            <h2 className="text-sm font-semibold text-neutral-600 dark:text-neutral-300">{t('playersHeading', { n: room.players.length })}</h2>
            <ul className="grid grid-cols-2 gap-2">
                {room.players.map(p => (
                    <li key={p.seat} className="flex items-center gap-2 rounded-lg border border-neutral-200 bg-neutral-50 px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800">
                        {p.seat === room.host_seat && <Crown className="h-4 w-4 flex-none text-amber-500" />}
                        <span className="truncate text-neutral-700 dark:text-neutral-200">{p.name || t('playerN', { n: p.seat + 1 })}</span>
                    </li>
                ))}
            </ul>

            {iAmHost ? (
                <button onClick={onStart} disabled={!room.can_start}
                    className="flex items-center justify-center gap-2 rounded-lg bg-primary-600 px-4 py-3 font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-40">
                    <Play className="h-5 w-5" />{room.can_start ? t('startGame') : t('needPlayers')}
                </button>
            ) : (
                <p className="text-center text-sm text-neutral-500 dark:text-neutral-400">{t('waitingHost')}</p>
            )}
        </div>
    );
}
