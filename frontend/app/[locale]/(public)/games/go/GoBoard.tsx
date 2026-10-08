"use client";

import { useId, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer } from '../_shared/pointer';
import { BOARD_INK, PieceShadow, StoneDefs, WoodDefs, WoodSurface } from '../_shared/BoardWood';
import { useRemovedPieces } from '../_shared/useRemovedPieces';
import type { HintsData } from '../_shared/wire';
import { SIZE, STARS, key, type Cell, type GBoard, type GColor } from './go-logic';

const CELL = 28;
const MARGIN = 22;
const W = (SIZE - 1) * CELL + 2 * MARGIN;
const H = W;
const R = 12.5; // 棋子半徑

function xy(col: number, row: number): [number, number] {
    return [MARGIN + col * CELL, MARGIN + (SIZE - 1 - row) * CELL]; // row 0 在下，無翻轉
}

export function GoBoard({
    board, lastMove, interactive, myColor, hints, boardLabel, onMove,
}: {
    board: GBoard;
    lastMove: Cell | null;
    interactive: boolean;
    myColor: GColor;
    /// server 給的禁著點（自殺 / 劫）—— 圍棋唯一需要規則判斷的提示
    hints: HintsData | null;
    boardLabel: string;
    onMove: (data: { at: Cell }) => void;
}) {
    const [confirm, setConfirm] = useState<Cell | null>(null);
    const uid = useId();
    const id = (name: string) => `${uid}-${name}`;
    const ghosts = useRemovedPieces(board, (a, b) => a === b); // 提子淡出

    const forbidden = hints?.forbidden ?? [];
    const isForbidden = (c: number, r: number) => forbidden.some(([fc, fr]) => fc === c && fr === r);

    const play = (c: number, r: number, touch: boolean) => {
        if (!interactive) return;
        if (board.has(key(c, r))) return;
        if (isForbidden(c, r)) return; // 提示層先擋掉，真正的判定仍在 server
        if (touch && !(confirm && confirm[0] === c && confirm[1] === r)) {
            setConfirm([c, r]);
            return;
        }
        setConfirm(null);
        onMove({ at: [c, r] });
    };

    const { cellProps } = useBoardCursor({
        cols: SIZE,
        rows: SIZE,
        enabled: interactive,
        onActivate: (c, r) => play(c, r, false),
        ariaLabel: (c, r) => {
            const stone = board.get(key(c, r));
            if (stone) return `${c + 1},${r + 1} ${stone === 'black' ? '●' : '○'}`;
            return `${c + 1},${r + 1}${isForbidden(c, r) ? ' ×' : ''}`;
        },
    });

    const lines: React.ReactNode[] = [];
    for (let i = 0; i < SIZE; i++) {
        const [hx1, hy1] = xy(0, i);
        const [hx2, hy2] = xy(SIZE - 1, i);
        lines.push(<line key={`h${i}`} x1={hx1} y1={hy1} x2={hx2} y2={hy2} />);
        const [vx1, vy1] = xy(i, 0);
        const [vx2, vy2] = xy(i, SIZE - 1);
        lines.push(<line key={`v${i}`} x1={vx1} y1={vy1} x2={vx2} y2={vy2} />);
    }

    const stone = (x: number, y: number, color: GColor) => (
        <>
            <PieceShadow id={id} x={x} y={y} r={R} />
            <circle cx={x} cy={y} r={R} fill={`url(#${id(color)})`} />
        </>
    );

    return (
        <svg viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg shadow-lg shadow-amber-950/30"
            role="group" aria-label={boardLabel}>
            <defs>
                <WoodDefs id={id} />
                <StoneDefs id={id} />
            </defs>

            <WoodSurface id={id} w={W} h={H} />

            <g stroke={BOARD_INK} strokeOpacity={0.75} strokeWidth={1} fill="none">{lines}</g>

            {STARS.map(([c, r], i) => {
                const [x, y] = xy(c, r);
                return <circle key={`s${i}`} cx={x} cy={y} r={3} fill={BOARD_INK} />;
            })}

            {ghosts.map(([k, color]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = xy(c, r);
                return <g key={`g${k}`} className="piece-capture" pointerEvents="none">{stone(x, y, color)}</g>;
            })}

            {Array.from(board.entries()).map(([k, color]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = xy(c, r);
                const isLast = !!lastMove && lastMove[0] === c && lastMove[1] === r;
                // 只有最後一手播落子動畫：新子 mount 時帶 class 才會播；中途進場／重連只有一顆會動
                return (
                    <g key={k} className={isLast ? 'piece-drop' : undefined}>
                        {stone(x, y, color)}
                        {isLast && <circle cx={x} cy={y} r={R * 0.42} fill="none" strokeWidth={1.6}
                            stroke={color === 'black' ? '#f5f5f4' : '#1c1917'} />}
                    </g>
                );
            })}

            {/* 禁著點（server 判的自殺 / 劫）：畫叉、且點不下去 */}
            {interactive && forbidden.map(([c, r], i) => {
                const [x, y] = xy(c, r);
                const d = 4.5;
                return (
                    <g key={`fb${i}`} pointerEvents="none" className="stroke-red-500/70" strokeWidth={1.6}>
                        <line x1={x - d} y1={y - d} x2={x + d} y2={y + d} />
                        <line x1={x - d} y1={y + d} x2={x + d} y2={y - d} />
                    </g>
                );
            })}

            {/* 待確認的落點（觸控） */}
            {confirm && !board.has(key(confirm[0], confirm[1])) && (() => {
                const [x, y] = xy(confirm[0], confirm[1]);
                return (
                    <g pointerEvents="none">
                        <circle cx={x} cy={y} r={R} opacity={0.5} fill={`url(#${id(myColor)})`} />
                        <circle cx={x} cy={y} r={R + 3.5} className="fill-none stroke-amber-400" strokeWidth={2} />
                    </g>
                );
            })()}

            {Array.from({ length: SIZE * SIZE }, (_, idx) => {
                const c = idx % SIZE;
                const r = Math.floor(idx / SIZE);
                // **有子的點也要留命中元素**：它同時是鍵盤游標的落腳處，
                // 跳過的話方向鍵移到有子的點就 focus 不過去，游標與焦點分家、之後的方向鍵全失效。
                // 落子本身仍由 play() 擋掉（已有子直接 return）。
                const [x, y] = xy(c, r);
                return <circle key={`hit${idx}`} cx={x} cy={y} r={CELL / 2 - 0.5}
                    fill="transparent"
                    className={interactive && !isForbidden(c, r) && !board.has(key(c, r))
                        ? 'cursor-pointer'
                        : undefined}
                    onPointerDown={(e) => { sound.warmup(); play(c, r, isTouchPointer(e)); }}
                    {...cellProps(c, r)} />;
            })}
        </svg>
    );
}
