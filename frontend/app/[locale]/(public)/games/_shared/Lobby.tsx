"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Loader2, Swords, Plus, LogIn, X } from 'lucide-react';
import { HowToPlayButton } from './HowToPlay';
import type { WireTable } from './wire';
import type { RoomPhase } from './useGameRoom';

// 大廳系列畫面（connecting / lobby / queued / hosting）；三遊戲共用，文案走 GameLobby namespace。
export function Lobby({
    phase, title, rules, tables, queuePos, hostedTableId, notice,
    onQuickMatch, onCreateTable, onJoinTable, onLeaveQueue, onCancelHost,
}: {
    phase: Extract<RoomPhase, 'connecting' | 'lobby' | 'queued' | 'hosting'>;
    title: string;
    rules: string[];
    tables: WireTable[];
    queuePos: number;
    hostedTableId: number | null;
    notice: string | null; // i18n key
    onQuickMatch: () => void;
    onCreateTable: (name: string) => void;
    onJoinTable: (id: number) => void;
    onLeaveQueue: () => void;
    onCancelHost: () => void;
}) {
    const t = useTranslations('GameLobby');
    const [name, setName] = useState('');

    if (phase === 'connecting') {
        return (
            <Centered>
                <Loader2 className="h-10 w-10 animate-spin text-primary-500" />
                <p className="text-sm text-neutral-500 dark:text-neutral-400">{t('connecting')}</p>
            </Centered>
        );
    }

    if (phase === 'queued') {
        return (
            <Centered>
                <Loader2 className="h-10 w-10 animate-spin text-primary-500" />
                <p className="text-sm text-neutral-600 dark:text-neutral-300">{t('queuedPosition', { position: queuePos })}</p>
                <button onClick={onLeaveQueue} className={secondaryBtn}>
                    <X className="h-4 w-4" />{t('leaveQueue')}
                </button>
            </Centered>
        );
    }

    if (phase === 'hosting') {
        return (
            <Centered>
                <Loader2 className="h-10 w-10 animate-spin text-primary-500" />
                <p className="text-sm text-neutral-600 dark:text-neutral-300">
                    {t('waitingOpponent')}{hostedTableId !== null && ` ${t('tableLabel', { id: hostedTableId })}`}
                </p>
                <button onClick={onCancelHost} className={secondaryBtn}>
                    <X className="h-4 w-4" />{t('cancel')}
                </button>
            </Centered>
        );
    }

    // lobby
    const waiting = tables.filter(tb => tb.status === 'waiting');
    return (
        <div className="mx-auto flex w-full max-w-lg flex-col gap-5 py-6">
            <h1 className="text-center text-2xl font-bold text-neutral-800 dark:text-neutral-100">{title}</h1>
            <HowToPlayButton title={title} rules={rules} />

            {notice && (
                <p className="rounded-lg border border-red-300 bg-red-50 px-3 py-2 text-center text-sm text-red-600 dark:border-red-800 dark:bg-red-950 dark:text-red-400">
                    {t(notice)}
                </p>
            )}

            <button
                onClick={onQuickMatch}
                className="flex items-center justify-center gap-2 rounded-lg bg-primary-600 px-4 py-3 font-medium text-white transition-colors hover:bg-primary-700"
            >
                <Swords className="h-5 w-5" />{t('quickMatch')}
            </button>

            <form
                onSubmit={e => { e.preventDefault(); onCreateTable(name.trim()); }}
                className="flex gap-2"
            >
                <input
                    value={name}
                    onChange={e => setName(e.target.value)}
                    maxLength={40}
                    placeholder={t('tableNamePlaceholder')}
                    className="min-w-0 flex-1 rounded-lg border border-neutral-300 bg-white px-3 py-2 text-sm text-neutral-800 transition-colors focus:border-primary-400 focus:outline-none dark:border-neutral-700 dark:bg-neutral-800 dark:text-neutral-100"
                />
                <button type="submit" className="flex flex-none items-center gap-2 rounded-lg border border-primary-400 px-3 py-2 text-sm text-primary-700 transition-colors hover:bg-primary-50 dark:text-primary-300 dark:hover:bg-primary-950">
                    <Plus className="h-4 w-4" />{t('createTable')}
                </button>
            </form>

            <div className="flex flex-col gap-2">
                <h2 className="text-sm font-semibold text-neutral-600 dark:text-neutral-300">{t('tablesHeading')}</h2>
                {waiting.length === 0 ? (
                    <p className="rounded-lg border border-dashed border-neutral-300 px-3 py-6 text-center text-sm text-neutral-400 dark:border-neutral-700">
                        {t('noTables')}
                    </p>
                ) : (
                    <ul className="flex flex-col gap-2">
                        {waiting.map(tb => (
                            <li
                                key={tb.id}
                                className="flex items-center justify-between rounded-lg border border-neutral-200 bg-neutral-50 px-3 py-2 dark:border-neutral-700 dark:bg-neutral-800"
                            >
                                <span className="truncate text-sm text-neutral-700 dark:text-neutral-200">{tb.name}</span>
                                <button
                                    onClick={() => onJoinTable(tb.id)}
                                    className="flex flex-none items-center gap-1.5 rounded-md bg-primary-600 px-3 py-1.5 text-sm text-white transition-colors hover:bg-primary-700"
                                >
                                    <LogIn className="h-4 w-4" />{t('join')}
                                </button>
                            </li>
                        ))}
                    </ul>
                )}
            </div>
        </div>
    );
}

const secondaryBtn = "mt-2 flex items-center gap-2 rounded-lg border border-neutral-300 px-4 py-2 text-sm text-neutral-600 transition-colors hover:bg-neutral-100 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800";

function Centered({ children }: { children: React.ReactNode }) {
    return (
        <div className="flex h-[calc(100svh-120px)] flex-col items-center justify-center gap-3">
            {children}
        </div>
    );
}
