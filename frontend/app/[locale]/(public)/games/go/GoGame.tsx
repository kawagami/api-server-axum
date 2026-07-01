"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { Hand } from 'lucide-react';
import { GameFrame } from '../_shared/GameFrame';
import { useGameRoom } from '../_shared/useGameRoom';
import { sound } from '../_shared/sound';
import { GoBoard } from './GoBoard';
import { applyPlay, emptyBoard, type Cell, type GBoard, type GColor } from './go-logic';

const SIDES = ['black', 'white'] as const;

interface GoMoveMade {
    at?: Cell; by: GColor; pass?: boolean; captured?: Cell[];
}

export default function GoGame() {
    const t = useTranslations('Go');

    const [board, setBoard] = useState<GBoard>(() => emptyBoard());
    const [lastMove, setLastMove] = useState<Cell | null>(null);

    const room = useGameRoom('go', SIDES, {
        onMatchFound: () => { setBoard(emptyBoard()); setLastMove(null); },
        onMoveMade: (data) => {
            const d = data as GoMoveMade;
            if (d.pass || !d.at) { sound.move(); setLastMove(null); return; }
            const captured = d.captured ?? [];
            if (captured.length) sound.capture(); else sound.move();
            setBoard(prev => applyPlay(prev, d.at!, d.by, captured));
            setLastMove(d.at);
        },
    });

    const myColor = room.myColor as GColor;
    const interactive = room.phase === 'playing' && room.turn === myColor && !room.pending;

    return (
        <GameFrame
            room={room}
            title={t('title')}
            rules={t.raw('rules') as string[]}
            sides={SIDES}
            sideLabel={(s) => t(s)}
            sideDotClass={(s) => (s === 'black' ? 'bg-neutral-900 dark:bg-neutral-50' : 'bg-white border border-neutral-400')}
            reasonLabel={(r) => t(`reason_${r}`)}
            extraControls={
                <button
                    onClick={() => room.actions.sendMove({ pass: true })}
                    disabled={!interactive}
                    className="flex items-center gap-2 rounded-lg border border-neutral-300 px-3 py-2 text-sm text-neutral-600 transition-colors hover:bg-neutral-100 disabled:cursor-not-allowed disabled:opacity-40 dark:border-neutral-700 dark:text-neutral-300 dark:hover:bg-neutral-800"
                >
                    <Hand className="h-4 w-4" />{t('pass')}
                </button>
            }
            board={
                <GoBoard
                    board={board}
                    lastMove={lastMove}
                    interactive={interactive}
                    onMove={room.actions.sendMove}
                />
            }
        />
    );
}
