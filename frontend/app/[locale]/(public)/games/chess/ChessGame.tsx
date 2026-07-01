"use client";

import { useEffect, useRef, useState } from 'react';
import { useTranslations } from 'next-intl';
import { GameFrame } from '../_shared/GameFrame';
import { useGameRoom } from '../_shared/useGameRoom';
import { sound } from '../_shared/sound';
import { ChessBoard } from './ChessBoard';
import {
    applyMove, initialBoard, key,
    type Board as BoardModel, type Side, type Square,
} from './chess-logic';

const SIDES = ['red', 'black'] as const;

interface MoveMade { from: Square; to: Square; }

export default function ChessGame() {
    const t = useTranslations('Chess');

    const [board, setBoard] = useState<BoardModel>(() => initialBoard());
    const [lastMove, setLastMove] = useState<MoveMade | null>(null);
    const [checkSide, setCheckSide] = useState<Side | null>(null);

    // board 鏡像供 WS callback（非 render）判斷吃子
    const boardRef = useRef(board);
    useEffect(() => { boardRef.current = board; }, [board]);

    const room = useGameRoom('chess', SIDES, {
        onMatchFound: () => {
            setBoard(initialBoard());
            setLastMove(null);
            setCheckSide(null);
        },
        onMoveMade: (data) => {
            const { from, to } = data as MoveMade;
            if (boardRef.current.has(key(to[0], to[1]))) sound.capture(); else sound.move();
            setBoard(prev => applyMove(prev, from, to));
            setLastMove({ from, to });
            setCheckSide(null);
        },
        onCheck: (data) => setCheckSide((data as { side: Side }).side),
    });

    const myColor = room.myColor as Side;
    const interactive = room.phase === 'playing' && room.turn === myColor && !room.pending;

    return (
        <GameFrame
            room={room}
            title={t('title')}
            rules={t.raw('rules') as string[]}
            sides={SIDES}
            sideLabel={(s) => t(s)}
            sideDotClass={(s) => (s === 'red' ? 'bg-red-600' : 'bg-neutral-900 dark:bg-neutral-100')}
            reasonLabel={(r) => t(`reason_${r}`)}
            extraStatus={checkSide && room.phase === 'playing'
                ? <span className="ml-2 font-semibold text-red-500">{t('check')}</span>
                : null}
            board={
                <ChessBoard
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
