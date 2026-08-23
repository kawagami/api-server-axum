"use client";

import { useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer } from '../_shared/pointer';
import { key, SIZE, type Cell, type GBoard } from './gomoku-logic';

const CELL = 36;
const MARGIN = 24;
const W = (SIZE - 1) * CELL + 2 * MARGIN;
const H = W;
const R = 15; // 棋子半徑

// 星位（hoshi）座標
const STARS: Cell[] = [[3, 3], [11, 3], [7, 7], [3, 11], [11, 11]];

function xy(col: number, row: number): [number, number] {
    // 無視角翻轉（棋盤對稱）；row 0 在下
    return [MARGIN + col * CELL, MARGIN + (SIZE - 1 - row) * CELL];
}

export function GomokuBoard({
    board, lastMove, interactive, myColor, boardLabel, onMove,
}: {
    board: GBoard;
    lastMove: Cell | null;
    interactive: boolean;
    myColor: 'black' | 'white';
    boardLabel: string;
    onMove: (data: { at: Cell }) => void;
}) {
    // 觸控先預覽再確認：手指的命中面積比格子大，直接落子的誤觸率很高，而落子不可撤回
    const [confirm, setConfirm] = useState<Cell | null>(null);

    const play = (c: number, r: number, touch: boolean) => {
        if (!interactive) return;
        if (board.has(key(c, r))) return; // 已有子
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
            return `${c + 1},${r + 1}${stone ? (stone === 'black' ? ' ●' : ' ○') : ''}`;
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

    return (
        <svg viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg bg-amber-50 dark:bg-neutral-900 shadow-sm"
            role="group" aria-label={boardLabel}>
            <g className="stroke-neutral-400 dark:stroke-neutral-600" strokeWidth={1.2} fill="none">{lines}</g>

            {/* 星位 */}
            {STARS.map(([c, r], i) => {
                const [x, y] = xy(c, r);
                return <circle key={`s${i}`} cx={x} cy={y} r={3.5} className="fill-neutral-400 dark:fill-neutral-600" />;
            })}

            {/* 棋子 */}
            {Array.from(board.entries()).map(([k, color]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = xy(c, r);
                const isLast = !!lastMove && lastMove[0] === c && lastMove[1] === r;
                return (
                    <g key={k}>
                        <circle cx={x} cy={y} r={R}
                            className={color === 'black'
                                ? 'fill-neutral-900 stroke-neutral-700'
                                : 'fill-neutral-50 stroke-neutral-400'}
                            strokeWidth={1} />
                        {isLast && <circle cx={x} cy={y} r={4}
                            className={color === 'black' ? 'fill-primary-400' : 'fill-primary-500'} />}
                    </g>
                );
            })}

            {/* 待確認的落點（觸控）：半透明預覽子 + 提示環，再點一次同一點才送出 */}
            {confirm && !board.has(key(confirm[0], confirm[1])) && (() => {
                const [x, y] = xy(confirm[0], confirm[1]);
                return (
                    <g pointerEvents="none">
                        <circle cx={x} cy={y} r={R} opacity={0.45}
                            className={myColor === 'black' ? 'fill-neutral-900' : 'fill-neutral-50 stroke-neutral-400'} />
                        <circle cx={x} cy={y} r={R + 4} className="fill-none stroke-amber-400" strokeWidth={2.5} />
                    </g>
                );
            })()}

            {/* 命中層（空點）：鍵盤可聚焦 + 指標按下 */}
            {Array.from({ length: SIZE * SIZE }, (_, idx) => {
                const c = idx % SIZE;
                const r = Math.floor(idx / SIZE);
                // **有子的點也要留命中元素**：它同時是鍵盤游標的落腳處，
                // 跳過的話方向鍵移到有子的點就 focus 不過去，游標與焦點分家、之後的方向鍵全失效。
                // 落子本身仍由 play() 擋掉（已有子直接 return）。
                const [x, y] = xy(c, r);
                return <circle key={`hit${idx}`} cx={x} cy={y} r={CELL / 2 - 1}
                    fill="transparent"
                    className={interactive && !board.has(key(c, r))
                        ? 'cursor-pointer focus:outline-2 focus:outline-primary-500'
                        : 'focus:outline-2 focus:outline-primary-500'}
                    onPointerDown={(e) => { sound.warmup(); play(c, r, isTouchPointer(e)); }}
                    {...cellProps(c, r)} />;
            })}
        </svg>
    );
}
