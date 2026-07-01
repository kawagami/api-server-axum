"use client";

import { useState } from 'react';
import { useTranslations } from 'next-intl';
import { GameFrame } from '../_shared/GameFrame';
import { useGameRoom } from '../_shared/useGameRoom';
import { sound } from '../_shared/sound';
import { GomokuBoard } from './GomokuBoard';
import { emptyBoard, placeStone, type Cell, type GBoard, type GColor } from './gomoku-logic';

const SIDES = ['black', 'white'] as const;

interface GMoveMade { at: Cell; by: GColor; }

export default function GomokuGame() {
    const t = useTranslations('Gomoku');

    const [board, setBoard] = useState<GBoard>(() => emptyBoard());
    const [lastMove, setLastMove] = useState<Cell | null>(null);

    const room = useGameRoom('gomoku', SIDES, {
        onMatchFound: () => { setBoard(emptyBoard()); setLastMove(null); },
        onMoveMade: (data) => {
            const { at, by } = data as GMoveMade;
            sound.move();
            setBoard(prev => placeStone(prev, at, by));
            setLastMove(at);
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
            sideDotClass={(s) => (s === 'black'
                ? 'bg-neutral-900 dark:bg-neutral-50'
                : 'bg-white border border-neutral-400')}
            reasonLabel={(r) => t(`reason_${r}`)}
            board={
                <GomokuBoard
                    board={board}
                    lastMove={lastMove}
                    interactive={interactive}
                    onMove={room.actions.sendMove}
                />
            }
        />
    );
}
