"use client";

import { useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer } from '../_shared/pointer';
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

    return (
        <svg viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg bg-amber-100 dark:bg-neutral-800 shadow-sm"
            role="group" aria-label={boardLabel}>
            <g className="stroke-neutral-500 dark:stroke-neutral-500" strokeWidth={1} fill="none">{lines}</g>

            {STARS.map(([c, r], i) => {
                const [x, y] = xy(c, r);
                return <circle key={`s${i}`} cx={x} cy={y} r={3} className="fill-neutral-500" />;
            })}

            {Array.from(board.entries()).map(([k, color]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = xy(c, r);
                const isLast = !!lastMove && lastMove[0] === c && lastMove[1] === r;
                return (
                    <g key={k}>
                        <circle cx={x} cy={y} r={R}
                            className={color === 'black' ? 'fill-neutral-900 stroke-neutral-700' : 'fill-neutral-50 stroke-neutral-400'}
                            strokeWidth={1} />
                        {isLast && <circle cx={x} cy={y} r={3.5}
                            className={color === 'black' ? 'fill-neutral-50' : 'fill-neutral-900'} />}
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
                        <circle cx={x} cy={y} r={R} opacity={0.45}
                            className={myColor === 'black' ? 'fill-neutral-900' : 'fill-neutral-50 stroke-neutral-400'} />
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
                        ? 'cursor-pointer focus:outline-2 focus:outline-primary-500'
                        : 'focus:outline-2 focus:outline-primary-500'}
                    onPointerDown={(e) => { sound.warmup(); play(c, r, isTouchPointer(e)); }}
                    {...cellProps(c, r)} />;
            })}
        </svg>
    );
}
