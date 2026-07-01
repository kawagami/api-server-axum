"use client";

import { useEffect, useRef, useState } from 'react';
import { useTranslations } from 'next-intl';
import { GameFrame } from '../_shared/GameFrame';
import { useGameRoom } from '../_shared/useGameRoom';
import { sound } from '../_shared/sound';
import { WesternChessBoard } from './WesternChessBoard';
import { applyMove, initialBoard, type Cell, type WBoard, type WColor, type WMoveMade } from './wc-logic';

const SIDES = ['white', 'black'] as const;

export default function WesternChessGame() {
    const t = useTranslations('WesternChess');

    const [board, setBoard] = useState<WBoard>(() => initialBoard());
    const [lastMove, setLastMove] = useState<{ from: Cell; to: Cell } | null>(null);
    const [checkSide, setCheckSide] = useState<WColor | null>(null);

    const boardRef = useRef(board);
    useEffect(() => { boardRef.current = board; }, [board]);

    const room = useGameRoom('western_chess', SIDES, {
        onMatchFound: () => { setBoard(initialBoard()); setLastMove(null); setCheckSide(null); },
        onMoveMade: (data) => {
            const d = data as WMoveMade;
            const { board: next, capture } = applyMove(boardRef.current, d);
            if (capture) sound.capture(); else sound.move();
            setBoard(next);
            setLastMove({ from: d.from, to: d.to });
            setCheckSide(null);
        },
        onCheck: (data) => setCheckSide((data as { side: WColor }).side),
    });

    const myColor = room.myColor as WColor;
    const interactive = room.phase === 'playing' && room.turn === myColor && !room.pending;

    return (
        <GameFrame
            room={room}
            title={t('title')}
            rules={t.raw('rules') as string[]}
            sides={SIDES}
            sideLabel={(s) => t(s)}
            sideDotClass={(s) => (s === 'white' ? 'bg-white border border-neutral-400' : 'bg-neutral-900 dark:bg-neutral-50')}
            reasonLabel={(r) => t(`reason_${r}`)}
            extraStatus={checkSide && room.phase === 'playing'
                ? <span className="ml-2 font-semibold text-red-500">{t('check')}</span>
                : null}
            board={
                <WesternChessBoard
                    board={board}
                    myColor={myColor}
                    lastMove={lastMove}
                    checkSide={checkSide}
                    interactive={interactive}
                    onMove={room.actions.sendMove}
                />
            }
        />
    );
}
