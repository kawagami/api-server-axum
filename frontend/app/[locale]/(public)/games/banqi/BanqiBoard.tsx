"use client";

import { useRef, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer, toViewBox } from '../_shared/pointer';
import type { HintsData } from '../_shared/wire';
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

// cellXY 的反函式：viewBox 座標 → 格（拖曳落子用）
function unproject(x: number, y: number): Cell | null {
    const col = Math.floor((x - MARGIN) / CELL);
    const row = ROWS - 1 - Math.floor((y - MARGIN) / CELL);
    if (col < 0 || col >= COLS || row < 0 || row >= ROWS) return null;
    return [col, row];
}

export function BanqiBoard({
    board, myColor, lastCells, interactive, hints, boardLabel, onMove,
}: {
    board: BBoard;
    myColor: BColor | null;   // 紅黑由首翻決定，未定為 null
    lastCells: Cell[];
    interactive: boolean;
    hints: HintsData | null;
    boardLabel: string;
    onMove: (data: BanqiIntent) => void;
}) {
    const [selected, setSelected] = useState<Cell | null>(null);
    // 觸控時翻子要先預覽再確認：翻子不可逆，誤觸的代價比走子高
    const [confirmFlip, setConfirmFlip] = useState<Cell | null>(null);
    const [drag, setDrag] = useState<{ from: Cell; x: number; y: number } | null>(null);
    const svgRef = useRef<SVGSVGElement>(null);

    // 選到子時 server 給的合法目標。flips 不另外標示 —— 所有蓋著的格都能翻，畫了只是噪音
    const targets: Cell[] = selected
        ? (hints?.moves?.[key(selected[0], selected[1])] as Cell[] | undefined) ?? []
        : [];

    const commitMove = (from: Cell, to: Cell) => {
        if (from[0] === to[0] && from[1] === to[1]) return;
        onMove({ action: 'move', from, to });
        setSelected(null);
    };

    // 點選（含鍵盤 Enter）；touch = true 時翻子需二次確認
    const activate = (c: number, r: number, touch = false) => {
        if (!interactive) return;
        const cell = board.get(key(c, r));
        if (cell?.hidden) {
            if (touch && !(confirmFlip && confirmFlip[0] === c && confirmFlip[1] === r)) {
                setConfirmFlip([c, r]);
                setSelected(null);
                return;
            }
            setConfirmFlip(null);
            onMove({ action: 'flip', at: [c, r] });
            setSelected(null);
            return;
        }
        setConfirmFlip(null);
        // 已翻開的己方子 → 選取
        if (cell && myColor && cell.color === myColor) { setSelected([c, r]); return; }
        // 目標格（空格或敵子）
        if (selected) commitMove(selected, [c, r]);
    };

    const onDown = (c: number, r: number) => (e: React.PointerEvent) => {
        sound.warmup();
        if (!interactive) return;
        const cell = board.get(key(c, r));
        const touch = isTouchPointer(e);
        if (cell && !cell.hidden && myColor && cell.color === myColor) {
            setSelected([c, r]);
            setConfirmFlip(null);
            if (!touch) {
                const [x, y] = cellXY(c, r);
                setDrag({ from: [c, r], x: x + CELL / 2, y: y + CELL / 2 });
            }
            return;
        }
        activate(c, r, touch);
    };

    const onSvgMove = (e: React.PointerEvent) => {
        if (!drag || !svgRef.current) return;
        const [x, y] = toViewBox(e, svgRef.current, W, H);
        setDrag({ ...drag, x, y });
    };

    const onSvgUp = (e: React.PointerEvent) => {
        if (!drag || !svgRef.current) return;
        const [x, y] = toViewBox(e, svgRef.current, W, H);
        const to = unproject(x, y);
        setDrag(null);
        if (to) commitMove(drag.from, to);
    };

    const dragOver = drag ? unproject(drag.x, drag.y) : null;
    const dragging = drag ? board.get(key(drag.from[0], drag.from[1])) : undefined;

    const { cellProps } = useBoardCursor({
        cols: COLS,
        rows: ROWS,
        enabled: interactive,
        onActivate: (c, r) => activate(c, r),
        ariaLabel: (c, r) => {
            const cell = board.get(key(c, r));
            if (!cell) return `${c + 1},${r + 1}`;
            if (cell.hidden) return `${c + 1},${r + 1} ?`;
            return `${c + 1},${r + 1}${cell.color && cell.kind ? ` ${kindChar(cell.color, cell.kind)}` : ''}`;
        },
    });

    const isSel = (c: number, r: number) => !!selected && selected[0] === c && selected[1] === r;
    const isLast = (c: number, r: number) => lastCells.some(([lc, lr]) => lc === c && lr === r);

    return (
        <svg ref={svgRef} viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            onPointerMove={onSvgMove} onPointerUp={onSvgUp} onPointerLeave={() => setDrag(null)}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg bg-amber-100 dark:bg-neutral-900 shadow-sm"
            role="group" aria-label={boardLabel}>
            {/* 格線 */}
            {Array.from({ length: COLS * ROWS }, (_, idx) => {
                const c = idx % COLS;
                const r = Math.floor(idx / COLS);
                const [x, y] = cellXY(c, r);
                const over = !!dragOver && dragOver[0] === c && dragOver[1] === r;
                return (
                    <g key={`g${idx}`}>
                        <rect x={x} y={y} width={CELL} height={CELL}
                            className="fill-none stroke-neutral-400 dark:stroke-neutral-600" strokeWidth={1} />
                        {over && <rect x={x + 1} y={y + 1} width={CELL - 2} height={CELL - 2}
                            className="fill-none stroke-primary-500/80" strokeWidth={3} />}
                    </g>
                );
            })}

            {/* 合法步提示（server 給的） */}
            {targets.map(([c, r], i) => {
                const [x, y] = cellXY(c, r);
                const occupied = board.has(key(c, r));
                return occupied
                    ? <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={R + 3}
                        className="fill-none stroke-emerald-500/80" strokeWidth={3} />
                    : <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={10} className="fill-emerald-500/45" />;
            })}

            {/* 棋子 */}
            {Array.from(board.entries()).map(([k, cell]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = cellXY(c, r);
                const cx = x + CELL / 2;
                const cy = y + CELL / 2;
                const sel = isSel(c, r);
                const last = isLast(c, r);
                const pendingFlip = !!confirmFlip && confirmFlip[0] === c && confirmFlip[1] === r;
                const isDragSrc = !!drag && drag.from[0] === c && drag.from[1] === r;
                return cell.hidden ? (
                    <g key={k}>
                        <circle cx={cx} cy={cy} r={R} className="fill-primary-600 dark:fill-primary-800 stroke-primary-800 dark:stroke-primary-950" strokeWidth={2} />
                        <circle cx={cx} cy={cy} r={R - 8} className="fill-none stroke-primary-300/50" strokeWidth={2} />
                        {/* 觸控待確認的翻子 */}
                        {pendingFlip && <circle cx={cx} cy={cy} r={R + 4} className="fill-none stroke-amber-400" strokeWidth={3} />}
                    </g>
                ) : (
                    <g key={k} opacity={isDragSrc ? 0.35 : 1}>
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

            {/* 跟著指標走的拖曳子 */}
            {drag && dragging && !dragging.hidden && (
                <g pointerEvents="none">
                    <circle cx={drag.x} cy={drag.y} r={R} className="fill-neutral-50/90 dark:fill-neutral-800/90 stroke-primary-500" strokeWidth={2} />
                    <text x={drag.x} y={drag.y} textAnchor="middle" dominantBaseline="central"
                        className={dragging.color === 'red' ? 'fill-red-700 dark:fill-red-400' : 'fill-neutral-900 dark:fill-neutral-100'}
                        style={{ fontSize: 34, fontWeight: 700 }}>
                        {dragging.color && dragging.kind ? kindChar(dragging.color, dragging.kind) : ''}
                    </text>
                </g>
            )}

            {/* 命中層（全 32 格）：鍵盤可聚焦 + 指標按下 */}
            {Array.from({ length: COLS * ROWS }, (_, idx) => {
                const c = idx % COLS;
                const r = Math.floor(idx / COLS);
                const [x, y] = cellXY(c, r);
                return <rect key={`hit${idx}`} x={x} y={y} width={CELL} height={CELL}
                    fill="transparent" className={interactive ? 'cursor-pointer focus:outline-2 focus:outline-primary-500' : ''}
                    onPointerDown={onDown(c, r)} {...cellProps(c, r)} />;
            })}
        </svg>
    );
}
