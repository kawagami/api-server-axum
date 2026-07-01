"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Loader2, Plus, LogIn, Users } from 'lucide-react';
import { HowToPlayButton } from '../_shared/HowToPlay';
import type { RoomSummary, RoomOptions } from './avalon-types';

export function AvalonLobby({
    connecting, rooms, notice, onCreate, onJoin,
}: {
    connecting: boolean;
    rooms: RoomSummary[];
    notice: string | null;
    onCreate: (roomName: string, nickname: string, options: RoomOptions) => void;
    onJoin: (roomId: number, nickname: string) => void;
}) {
    const t = useTranslations('Avalon');
    const [nickname, setNickname] = useState('');
    const [roomName, setRoomName] = useState('');
    const [mordred, setMordred] = useState(false);
    const [oberon, setOberon] = useState(false);

    if (connecting) {
        return (
            <div className="flex h-[calc(100svh-120px)] flex-col items-center justify-center gap-3">
                <Loader2 className="h-10 w-10 animate-spin text-primary-500" />
                <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('connecting')}</p>
            </div>
        );
    }

    return (
        <div className="mx-auto flex w-full max-w-lg flex-col gap-5 py-6">
            <h1 className="text-center text-2xl font-bold text-neutral-800 dark:text-neutral-100">{t('lobbyTitle')}</h1>
            <HowToPlayButton title={t('title')} rules={t.raw('rules') as string[]} />

            {notice && (
                <p className="rounded-lg border border-red-300 bg-red-50 px-3 py-2 text-center text-sm text-red-600 dark:border-red-800 dark:bg-red-950 dark:text-red-400">
                    {t(notice)}
                </p>
            )}

            <label className="flex flex-col gap-1 text-sm">
                <span className="font-medium text-neutral-600 dark:text-neutral-300">{t('nickname')}</span>
                <input
                    value={nickname} onChange={e => setNickname(e.target.value)} maxLength={20}
                    placeholder={t('nicknamePlaceholder')}
                    className="rounded-lg border border-neutral-300 bg-white px-3 py-2 text-neutral-800 focus:border-primary-400 focus:outline-none dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
                />
            </label>

            {/* 建房 */}
            <div className="flex flex-col gap-2 rounded-lg border border-neutral-200 p-3 dark:border-neutral-700">
                <input
                    value={roomName} onChange={e => setRoomName(e.target.value)} maxLength={40}
                    placeholder={t('roomNamePlaceholder')}
                    className="rounded-lg border border-neutral-300 bg-white px-3 py-2 text-sm text-neutral-800 focus:border-primary-400 focus:outline-none dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
                />
                <div className="flex gap-4 text-sm text-neutral-600 dark:text-neutral-300">
                    <label className="flex items-center gap-1.5">
                        <input type="checkbox" checked={mordred} onChange={e => setMordred(e.target.checked)} />
                        {t('optMordred')}
                    </label>
                    <label className="flex items-center gap-1.5">
                        <input type="checkbox" checked={oberon} onChange={e => setOberon(e.target.checked)} />
                        {t('optOberon')}
                    </label>
                </div>
                <button
                    onClick={() => onCreate(roomName.trim(), nickname.trim(), { mordred, oberon })}
                    className="flex items-center justify-center gap-2 rounded-lg bg-primary-600 px-4 py-2 font-medium text-white transition-colors hover:bg-primary-700"
                >
                    <Plus className="h-5 w-5" />{t('createRoom')}
                </button>
            </div>

            {/* 房列表 */}
            <div className="flex flex-col gap-2">
                <h2 className="text-sm font-semibold text-neutral-600 dark:text-neutral-300">{t('roomsHeading')}</h2>
                {rooms.length === 0 ? (
                    <p className="rounded-lg border border-dashed border-neutral-300 px-3 py-6 text-center text-sm text-neutral-400 dark:border-neutral-700">
                        {t('noRooms')}
                    </p>
                ) : (
                    <ul className="flex flex-col gap-2">
                        {rooms.map(r => {
                            const joinable = r.status === 'waiting' && r.players < r.max;
                            return (
                                <li key={r.id} className="flex items-center justify-between rounded-lg border border-neutral-200 bg-neutral-50 px-3 py-2 dark:border-neutral-700 dark:bg-neutral-800">
                                    <span className="flex items-center gap-2 truncate text-sm text-neutral-700 dark:text-neutral-200">
                                        <span className="truncate">{r.name}</span>
                                        <span className="flex flex-none items-center gap-1 text-xs text-neutral-400">
                                            <Users className="h-3.5 w-3.5" />{r.players}/{r.max}
                                        </span>
                                    </span>
                                    <button
                                        onClick={() => onJoin(r.id, nickname.trim())}
                                        disabled={!joinable}
                                        className="flex flex-none items-center gap-1.5 rounded-md bg-primary-600 px-3 py-1.5 text-sm text-white transition-colors hover:bg-primary-700 disabled:cursor-not-allowed disabled:opacity-40"
                                    >
                                        <LogIn className="h-4 w-4" />{joinable ? t('join') : t(`status_${r.status}`)}
                                    </button>
                                </li>
                            );
                        })}
                    </ul>
                )}
            </div>
        </div>
    );
}
