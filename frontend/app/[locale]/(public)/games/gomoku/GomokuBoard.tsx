"use client";

import { sound } from '../_shared/sound';
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
    board, lastMove, interactive, onMove,
}: {
    board: GBoard;
    lastMove: Cell | null;
    interactive: boolean;
    onMove: (data: { at: Cell }) => void;
}) {
    const onPoint = (c: number, r: number) => {
        sound.warmup();
        if (!interactive) return;
        if (board.has(key(c, r))) return; // 已有子
        onMove({ at: [c, r] });
    };

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
            className="max-h-full max-w-full select-none rounded-lg bg-amber-50 dark:bg-neutral-900 shadow" role="img">
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

            {/* 點擊命中層（空點，置頂） */}
            {interactive && Array.from({ length: SIZE * SIZE }, (_, idx) => {
                const c = idx % SIZE;
                const r = Math.floor(idx / SIZE);
                if (board.has(key(c, r))) return null;
                const [x, y] = xy(c, r);
                return <circle key={`hit${idx}`} cx={x} cy={y} r={CELL / 2 - 1}
                    fill="transparent" className="cursor-pointer" onClick={() => onPoint(c, r)} />;
            })}
        </svg>
    );
}
