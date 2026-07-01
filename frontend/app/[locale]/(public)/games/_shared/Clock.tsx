"use client";

import { useEffect, useState } from 'react';

function fmt(ms: number): string {
    const total = Math.max(0, Math.ceil(ms / 1000));
    const m = Math.floor(total / 60);
    const s = total % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
}

const TICK = 250;

// server 在每次 move_made 帶權威時鐘；兩步之間前端本地遞減「當前行棋方」做平滑顯示。
// 重設靠父層用 key 重掛載（避免 effect 內同步 setState）。判輸仍由 server 權威。
export function Clock({
    label, dotClass, baseMs, running,
}: {
    label: string;
    dotClass: string;   // 標示該方的小圓點配色（各遊戲傳入）
    baseMs: number;
    running: boolean;
}) {
    const [displayMs, setDisplayMs] = useState(baseMs);

    useEffect(() => {
        if (!running) return;
        const id = setInterval(() => setDisplayMs(d => Math.max(0, d - TICK)), TICK);
        return () => clearInterval(id);
    }, [running]);

    const low = displayMs <= 30_000;

    return (
        <div
            className={[
                'flex items-center justify-between gap-3 rounded-lg border px-4 py-2 transition-colors',
                running
                    ? 'border-primary-400 bg-primary-50 dark:border-primary-600 dark:bg-primary-950'
                    : 'border-neutral-200 bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800',
            ].join(' ')}
        >
            <span className="flex items-center gap-2 text-sm font-medium text-neutral-700 dark:text-neutral-200">
                <span className={`inline-block h-3 w-3 rounded-full ${dotClass}`} />
                {label}
            </span>
            <span
                className={[
                    'font-mono text-lg tabular-nums',
                    low ? 'text-red-600 dark:text-red-400' : 'text-neutral-800 dark:text-neutral-100',
                    running && low ? 'animate-pulse' : '',
                ].join(' ')}
            >
                {fmt(displayMs)}
            </span>
        </div>
    );
}
