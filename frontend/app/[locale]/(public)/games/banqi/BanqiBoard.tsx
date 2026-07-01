"use client";

import { useState } from 'react';
import { sound } from '../_shared/sound';
import { COLS, ROWS, key, kindChar, type BBoard, type BColor, type Cell } from './banqi-logic';

const CELL = 74;
const MARGIN = 12;
const W = COLS * CELL + 2 * MARGIN;
const H = ROWS * CELL + 2 * MARGIN;
const R = 31; // 棋子半徑

export type BanqiIntent =
    | { action: 'flip'; at: Cell }
    | { action: 'move'; from: Cell; to: Cell };

// 格左上角座標（row 0 在下）
function cellXY(col: number, row: number): [number, number] {
    return [MARGIN + col * CELL, MARGIN + (ROWS - 1 - row) * CELL];
}

export function BanqiBoard({
    board, myColor, lastCells, interactive, onMove,
}: {
    board: BBoard;
    myColor: BColor | null;   // 紅黑由首翻決定，未定為 null
    lastCells: Cell[];
    interactive: boolean;
    onMove: (data: BanqiIntent) => void;
}) {
    const [selected, setSelected] = useState<Cell | null>(null);

    const onCell = (c: number, r: number) => {
        sound.warmup();
        if (!interactive) return;
        const cell = board.get(key(c, r));
        if (cell?.hidden) { onMove({ action: 'flip', at: [c, r] }); setSelected(null); return; }
        // cell 此處為已翻開或空格
        if (cell && myColor && cell.color === myColor) { setSelected([c, r]); return; }
        // 目標格（空格或敵子）
        if (selected) { onMove({ action: 'move', from: selected, to: [c, r] }); setSelected(null); }
    };

    const isSel = (c: number, r: number) => !!selected && selected[0] === c && selected[1] === r;
    const isLast = (c: number, r: number) => lastCells.some(([lc, lr]) => lc === c && lr === r);

    return (
        <svg viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            className="max-h-full max-w-full select-none rounded-lg bg-amber-100 dark:bg-neutral-900 shadow" role="img">
            {/* 格線 */}
            {Array.from({ length: COLS * ROWS }, (_, idx) => {
                const c = idx % COLS;
                const r = Math.floor(idx / COLS);
                const [x, y] = cellXY(c, r);
                return <rect key={`g${idx}`} x={x} y={y} width={CELL} height={CELL}
                    className="fill-none stroke-neutral-400 dark:stroke-neutral-600" strokeWidth={1} />;
            })}

            {/* 棋子 */}
            {Array.from(board.entries()).map(([k, cell]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = cellXY(c, r);
                const cx = x + CELL / 2;
                const cy = y + CELL / 2;
                const sel = isSel(c, r);
                const last = isLast(c, r);
                return cell.hidden ? (
                    <g key={k}>
                        <circle cx={cx} cy={cy} r={R} className="fill-primary-600 dark:fill-primary-800 stroke-primary-800 dark:stroke-primary-950" strokeWidth={2} />
                        <circle cx={cx} cy={cy} r={R - 8} className="fill-none stroke-primary-300/50" strokeWidth={2} />
                    </g>
                ) : (
                    <g key={k}>
                        <circle cx={cx} cy={cy} r={R} className="fill-neutral-50 dark:fill-neutral-800" />
                        <circle cx={cx} cy={cy} r={R}
                            className={sel ? 'fill-none stroke-primary-500'
                                : last ? 'fill-none stroke-primary-400/70'
                                    : cell.color === 'red' ? 'fill-none stroke-red-600/70'
                                        : 'fill-none stroke-neutral-700 dark:stroke-neutral-300'}
                            strokeWidth={sel ? 3 : last ? 2.5 : 1.5} strokeDasharray={last && !sel ? '4 3' : undefined} />
                        <text x={cx} y={cy} textAnchor="middle" dominantBaseline="central"
                            className={cell.color === 'red' ? 'fill-red-700 dark:fill-red-400' : 'fill-neutral-900 dark:fill-neutral-100'}
                            style={{ fontSize: 34, fontWeight: 700 }}>
                            {cell.color && cell.kind ? kindChar(cell.color, cell.kind) : ''}
                        </text>
                    </g>
                );
            })}

            {/* lastMove 空格標記（移動後 from 變空格也標一下） */}
            {lastCells.map(([c, r], i) => {
                if (board.has(key(c, r))) return null;
                const [x, y] = cellXY(c, r);
                return <circle key={`le${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={6}
                    className="fill-primary-400/50" />;
            })}

            {/* 點擊命中層（全 32 格） */}
            {interactive && Array.from({ length: COLS * ROWS }, (_, idx) => {
                const c = idx % COLS;
                const r = Math.floor(idx / COLS);
                const [x, y] = cellXY(c, r);
                return <rect key={`hit${idx}`} x={x} y={y} width={CELL} height={CELL}
                    fill="transparent" className="cursor-pointer" onClick={() => onCell(c, r)} />;
            })}
        </svg>
    );
}
