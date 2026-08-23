"use client";

import { useRef, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer, toViewBox } from '../_shared/pointer';
import type { HintsData } from '../_shared/wire';
import { key, pieceChar, type Board as BoardModel, type Side, type Square } from './chess-logic';

const CELL = 64;
const MARGIN = 36;
const W = 8 * CELL + 2 * MARGIN;
const H = 9 * CELL + 2 * MARGIN;
const R = 26; // 棋子半徑

// 棋盤座標 → SVG 像素（依我方顏色翻轉，己方永遠在下）
function project(col: number, row: number, myColor: Side): [number, number] {
    if (myColor === 'red') return [MARGIN + col * CELL, MARGIN + (9 - row) * CELL];
    return [MARGIN + (8 - col) * CELL, MARGIN + row * CELL]; // 黑方視角：兩軸翻轉
}

// project 的反函式：SVG 像素 → 最近的棋盤交叉點（拖曳落子用）。超出盤面回 null
function unproject(x: number, y: number, myColor: Side): Square | null {
    const i = Math.round((x - MARGIN) / CELL);
    const j = Math.round((y - MARGIN) / CELL);
    const col = myColor === 'red' ? i : 8 - i;
    const row = myColor === 'red' ? 9 - j : j;
    if (col < 0 || col > 8 || row < 0 || row > 9) return null;
    return [col, row];
}

export function ChessBoard({
    board, myColor, lastMove, checkSide, interactive, hints, boardLabel, onMove,
}: {
    board: BoardModel;
    myColor: Side;
    lastMove: { from: Square; to: Square } | null;
    checkSide: Side | null;
    interactive: boolean;
    hints: HintsData | null;
    boardLabel: string;
    onMove: (data: { from: Square; to: Square }) => void;
}) {
    const [selected, setSelected] = useState<Square | null>(null);
    // 拖曳中的來源格與指標位置（viewBox 座標）
    const [drag, setDrag] = useState<{ from: Square; x: number; y: number } | null>(null);
    const svgRef = useRef<SVGSVGElement>(null);

    // 選到子時 server 給的合法目標（純提示；判定仍在後端）
    const targets: Square[] = selected
        ? (hints?.moves?.[key(selected[0], selected[1])] as Square[] | undefined) ?? []
        : [];
    const isTarget = (c: number, r: number) => targets.some(([tc, tr]) => tc === c && tr === r);

    const commit = (from: Square, to: Square) => {
        if (from[0] === to[0] && from[1] === to[1]) return;
        onMove({ from, to });
        setSelected(null);
    };

    // 點選（含鍵盤 Enter）：先選己方子，再點目標
    const activate = (c: number, r: number) => {
        if (!interactive) return;
        const piece = board.get(key(c, r));
        if (piece && piece.side === myColor) { setSelected([c, r]); return; }
        if (selected) commit(selected, [c, r]);
    };

    const onDown = (c: number, r: number) => (e: React.PointerEvent) => {
        sound.warmup();
        if (!interactive) return;
        const piece = board.get(key(c, r));
        if (piece && piece.side === myColor) {
            setSelected([c, r]);
            // 觸控不進拖曳模式：手指按住會與頁面捲動打架，觸控維持「點選 → 點目標」
            if (!isTouchPointer(e)) {
                const [x, y] = project(c, r, myColor);
                setDrag({ from: [c, r], x, y });
            }
            return;
        }
        if (selected) commit(selected, [c, r]);
    };

    const onSvgMove = (e: React.PointerEvent) => {
        if (!drag || !svgRef.current) return;
        const [x, y] = toViewBox(e, svgRef.current, W, H);
        setDrag({ ...drag, x, y });
    };

    const onSvgUp = (e: React.PointerEvent) => {
        if (!drag || !svgRef.current) return;
        const [x, y] = toViewBox(e, svgRef.current, W, H);
        const to = unproject(x, y, myColor);
        setDrag(null);
        if (to) commit(drag.from, to);
    };

    const dragOver = drag ? unproject(drag.x, drag.y, myColor) : null;

    const { cellProps } = useBoardCursor({
        cols: 9,
        rows: 10,
        enabled: interactive,
        flipped: myColor !== 'red',
        onActivate: activate,
        ariaLabel: (c, r) => {
            const p = board.get(key(c, r));
            return `${c + 1},${r + 1}${p ? ` ${pieceChar(p)}` : ''}`;
        },
    });

    const lines: React.ReactNode[] = [];
    for (let r = 0; r < 10; r++) {
        const [x1, y1] = project(0, r, myColor);
        const [x2, y2] = project(8, r, myColor);
        lines.push(<line key={`h${r}`} x1={x1} y1={y1} x2={x2} y2={y2} />);
    }
    for (let c = 0; c < 9; c++) {
        if (c === 0 || c === 8) {
            const [x1, y1] = project(c, 0, myColor);
            const [x2, y2] = project(c, 9, myColor);
            lines.push(<line key={`v${c}`} x1={x1} y1={y1} x2={x2} y2={y2} />);
        } else {
            const [ax1, ay1] = project(c, 0, myColor);
            const [ax2, ay2] = project(c, 4, myColor);
            const [bx1, by1] = project(c, 5, myColor);
            const [bx2, by2] = project(c, 9, myColor);
            lines.push(<line key={`v${c}a`} x1={ax1} y1={ay1} x2={ax2} y2={ay2} />);
            lines.push(<line key={`v${c}b`} x1={bx1} y1={by1} x2={bx2} y2={by2} />);
        }
    }
    const palace: [Square, Square][] = [
        [[3, 0], [5, 2]], [[5, 0], [3, 2]],
        [[3, 9], [5, 7]], [[5, 9], [3, 7]],
    ];
    palace.forEach(([a, b], i) => {
        const [x1, y1] = project(a[0], a[1], myColor);
        const [x2, y2] = project(b[0], b[1], myColor);
        lines.push(<line key={`p${i}`} x1={x1} y1={y1} x2={x2} y2={y2} />);
    });

    const [riverX, riverY] = project(4, 4, myColor);
    const dragging = board.get(drag ? key(drag.from[0], drag.from[1]) : '');

    return (
        <svg ref={svgRef} viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            onPointerMove={onSvgMove} onPointerUp={onSvgUp} onPointerLeave={() => setDrag(null)}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg bg-amber-50 dark:bg-neutral-900 shadow-sm"
            role="group" aria-label={boardLabel}>
            <g className="stroke-neutral-400 dark:stroke-neutral-600" strokeWidth={1.5} fill="none">{lines}</g>

            <text x={riverX} y={riverY + CELL / 2} textAnchor="middle" dominantBaseline="middle"
                className="fill-neutral-400 dark:fill-neutral-600" style={{ fontSize: 22, letterSpacing: 8 }}>
                楚河　漢界
            </text>

            {lastMove && [lastMove.from, lastMove.to].map((sq, i) => {
                const [x, y] = project(sq[0], sq[1], myColor);
                return <circle key={`lm${i}`} cx={x} cy={y} r={R + 3}
                    className="fill-none stroke-primary-400/70" strokeWidth={2} strokeDasharray="4 3" />;
            })}

            {/* 合法步提示（server 給的，前端不自己算規則） */}
            {targets.map(([c, r], i) => {
                const [x, y] = project(c, r, myColor);
                const occupied = board.has(key(c, r));
                return occupied
                    ? <circle key={`ht${i}`} cx={x} cy={y} r={R + 2} className="fill-none stroke-emerald-500/80" strokeWidth={3} />
                    : <circle key={`ht${i}`} cx={x} cy={y} r={8} className="fill-emerald-500/45" />;
            })}

            {/* 拖曳中的落點提示 */}
            {dragOver && (() => {
                const [x, y] = project(dragOver[0], dragOver[1], myColor);
                return <circle cx={x} cy={y} r={R + 4} className="fill-none stroke-primary-500/70" strokeWidth={2} />;
            })()}

            {Array.from(board.entries()).map(([k, piece]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = project(c, r, myColor);
                const isSel = !!selected && selected[0] === c && selected[1] === r;
                const inCheck = checkSide === piece.side && piece.type === 'general';
                const isDragSrc = !!drag && drag.from[0] === c && drag.from[1] === r;
                return (
                    <g key={k} className={inCheck ? 'animate-pulse' : undefined} opacity={isDragSrc ? 0.35 : 1}>
                        <circle cx={x} cy={y} r={R} className="fill-neutral-50 dark:fill-neutral-800" />
                        <circle cx={x} cy={y} r={R}
                            className={isSel ? 'fill-none stroke-primary-500'
                                : inCheck ? 'fill-none stroke-red-500'
                                    : piece.side === 'red' ? 'fill-none stroke-red-600/70'
                                        : 'fill-none stroke-neutral-700 dark:stroke-neutral-300'}
                            strokeWidth={isSel || inCheck ? 3 : 1.5} />
                        <text x={x} y={y} textAnchor="middle" dominantBaseline="central"
                            className={piece.side === 'red' ? 'fill-red-700 dark:fill-red-400' : 'fill-neutral-900 dark:fill-neutral-100'}
                            style={{ fontSize: 30, fontWeight: 700 }}>
                            {pieceChar(piece)}
                        </text>
                    </g>
                );
            })}

            {/* 跟著指標走的拖曳子 */}
            {drag && dragging && (
                <g pointerEvents="none">
                    <circle cx={drag.x} cy={drag.y} r={R} className="fill-neutral-50/90 dark:fill-neutral-800/90 stroke-primary-500" strokeWidth={2} />
                    <text x={drag.x} y={drag.y} textAnchor="middle" dominantBaseline="central"
                        className={dragging.side === 'red' ? 'fill-red-700 dark:fill-red-400' : 'fill-neutral-900 dark:fill-neutral-100'}
                        style={{ fontSize: 30, fontWeight: 700 }}>
                        {pieceChar(dragging)}
                    </text>
                </g>
            )}

            {/* 命中層：鍵盤可聚焦（roving tabindex）+ 指標按下 */}
            {Array.from({ length: 90 }, (_, idx) => {
                const c = idx % 9;
                const r = Math.floor(idx / 9);
                const [x, y] = project(c, r, myColor);
                return <circle key={`hit${idx}`} cx={x} cy={y} r={CELL / 2 - 2}
                    fill="transparent" className={interactive ? 'cursor-pointer focus:outline-2 focus:outline-primary-500' : ''}
                    onPointerDown={onDown(c, r)} {...cellProps(c, r)} />;
            })}
        </svg>
    );
}
