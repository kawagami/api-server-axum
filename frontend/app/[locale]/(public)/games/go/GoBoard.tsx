"use client";

import { sound } from '../_shared/sound';
import { SIZE, STARS, key, type Cell, type GBoard } from './go-logic';

const CELL = 28;
const MARGIN = 22;
const W = (SIZE - 1) * CELL + 2 * MARGIN;
const H = W;
const R = 12.5; // 棋子半徑

function xy(col: number, row: number): [number, number] {
    return [MARGIN + col * CELL, MARGIN + (SIZE - 1 - row) * CELL]; // row 0 在下，無翻轉
}

export function GoBoard({
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
        if (board.has(key(c, r))) return;
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
            className="max-h-full max-w-full select-none rounded-lg bg-amber-100 dark:bg-neutral-800 shadow" role="img">
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

            {interactive && Array.from({ length: SIZE * SIZE }, (_, idx) => {
                const c = idx % SIZE;
                const r = Math.floor(idx / SIZE);
                if (board.has(key(c, r))) return null;
                const [x, y] = xy(c, r);
                return <circle key={`hit${idx}`} cx={x} cy={y} r={CELL / 2 - 0.5}
                    fill="transparent" className="cursor-pointer" onClick={() => onPoint(c, r)} />;
            })}
        </svg>
    );
}
