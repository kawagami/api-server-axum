"use client";

import { useRef, useState } from 'react';
import { useTranslations } from 'next-intl';
import { GameFrame } from '../_shared/GameFrame';
import { useGameRoom } from '../_shared/useGameRoom';
import { sound } from '../_shared/sound';
import { BanqiBoard } from './BanqiBoard';
import {
    applyFlip, applyMove, initialBoard,
    type BBoard, type BColor, type BKind, type Cell,
} from './banqi-logic';

const SIDES = ['first', 'second'] as const;
const opp = (c: BColor): BColor => (c === 'red' ? 'black' : 'red');

interface FlipMade { action: 'flip'; at: Cell; piece: { color: BColor; kind: BKind }; turn: string; }
interface MoveMade { action: 'move'; from: Cell; to: Cell; captured: { color: BColor; kind: BKind } | null; }

export default function BanqiGame() {
    const t = useTranslations('Banqi');

    const [board, setBoard] = useState<BBoard>(() => initialBoard());
    const [lastCells, setLastCells] = useState<Cell[]>([]);
    const [myBanqiColor, setMyBanqiColor] = useState<BColor | null>(null);

    const seatRef = useRef<string>('first'); // 我方座位（match_found）
    const colorRef = useRef<BColor | null>(null); // 我方紅黑（首翻定）

    const room = useGameRoom('banqi', SIDES, {
        onMatchFound: (color) => {
            seatRef.current = color;
            colorRef.current = null;
            setMyBanqiColor(null);
            setBoard(initialBoard());
            setLastCells([]);
        },
        onMoveMade: (data) => {
            const d = data as (FlipMade | MoveMade) & { turn: string };
            if (d.action === 'flip') {
                sound.move();
                setBoard(prev => applyFlip(prev, d.at, d.piece.color, d.piece.kind));
                setLastCells([d.at]);
                if (colorRef.current === null) {
                    // 走子方 = 結果 turn 的對方；我若是走子方則執翻出色，否則執對色
                    const moverSeat = d.turn === SIDES[0] ? SIDES[1] : SIDES[0];
                    const mine = moverSeat === seatRef.current ? d.piece.color : opp(d.piece.color);
                    colorRef.current = mine;
                    setMyBanqiColor(mine);
                }
            } else {
                if (d.captured) sound.capture(); else sound.move();
                setBoard(prev => applyMove(prev, d.from, d.to));
                setLastCells([d.from, d.to]);
            }
        },
    });

    const interactive = room.phase === 'playing' && room.turn === room.myColor && !room.pending;

    return (
        <GameFrame
            room={room}
            title={t('title')}
            rules={t.raw('rules') as string[]}
            sides={SIDES}
            sideLabel={(s) => t(s)}
            sideDotClass={(s) => (s === 'first' ? 'bg-primary-600' : 'bg-neutral-500')}
            reasonLabel={(r) => t(`reason_${r}`)}
            board={
                <BanqiBoard
                    board={board}
                    myColor={myBanqiColor}
                    lastCells={lastCells}
                    interactive={interactive}
                    onMove={room.actions.sendMove}
                />
            }
        />
    );
}
