"use client";

import { useTranslations } from 'next-intl';
import { Crown, Play, LogOut } from 'lucide-react';
import { AvalonChat } from './AvalonChat';
import type { ChatEntry, RoomUpdateData } from './avalon-types';

export function AvalonRoom({
    room, iAmHost, chat, onStart, onLeave, onChat,
}: {
    room: RoomUpdateData;
    iAmHost: boolean;
    chat: ChatEntry[];
    onStart: () => void;
    onLeave: () => void;
    onChat: (text: string) => void;
}) {
    const t = useTranslations('Avalon');

    return (
        <div className="mx-auto flex h-[calc(100svh-120px)] w-full max-w-lg flex-col gap-3 py-4">
            <div className="flex items-center justify-between">
                <h1 className="truncate text-xl font-bold text-neutral-800 dark:text-neutral-100">{room.name}</h1>
                <button onClick={onLeave} className="flex flex-none items-center gap-1.5 rounded-lg border border-neutral-300 px-3 py-1.5 text-sm text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800">
                    <LogOut className="h-4 w-4" />{t('leave')}
                </button>
            </div>

            <div className="flex gap-2 text-xs">
                {room.options?.mordred && <span className="rounded-full bg-primary-100 px-2 py-0.5 text-primary-700 dark:bg-primary-900 dark:text-primary-200">{t('optMordred')}</span>}
                {room.options?.oberon && <span className="rounded-full bg-primary-100 px-2 py-0.5 text-primary-700 dark:bg-primary-900 dark:text-primary-200">{t('optOberon')}</span>}
            </div>

            <div className="flex flex-col gap-2">
                <h2 className="text-sm font-semibold text-neutral-600 dark:text-neutral-300">
                    {t('playersHeading', { n: room.players.length })}
                </h2>
                <ul className="grid grid-cols-2 gap-2">
                    {room.players.map(p => (
                        <li key={p.seat} className="flex items-center gap-2 rounded-lg border border-neutral-200 bg-neutral-50 px-3 py-2 text-sm dark:border-neutral-700 dark:bg-neutral-800">
                            {p.seat === room.host_seat && <Crown className="h-4 w-4 flex-none text-amber-500" />}
                            <span className="truncate text-neutral-700 dark:text-neutral-200">{p.name || t('playerN', { n: p.seat + 1 })}</span>
                        </li>
                    ))}
                </ul>
            </div>

            {iAmHost ? (
                <button
                    onClick={onStart}
                    disabled={!room.can_start}
                    className="flex items-center justify-center gap-2 rounded-lg bg-primary-600 px-4 py-3 font-medium text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-40"
                >
                    <Play className="h-5 w-5" />{room.can_start ? t('startGame') : t('needMorePlayers')}
                </button>
            ) : (
                <p className="text-center text-sm text-neutral-500 dark:text-neutral-400">{t('waitingHost')}</p>
            )}

            <AvalonChat chat={chat} onSend={onChat} />
        </div>
    );
}
