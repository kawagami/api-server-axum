"use client";

import { useRef, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer, toViewBox } from '../_shared/pointer';
import type { HintsData } from '../_shared/wire';
import { SIZE, glyph, key, type Cell, type WBoard, type WColor, type WKind } from './wc-logic';

const CELL = 64;
const MARGIN = 10;
const W = SIZE * CELL + 2 * MARGIN;
const H = W;

// 棋格左上角（依我方顏色翻轉）
function origin(col: number, row: number, my: WColor): [number, number] {
    if (my === 'white') return [MARGIN + col * CELL, MARGIN + (SIZE - 1 - row) * CELL];
    return [MARGIN + (SIZE - 1 - col) * CELL, MARGIN + row * CELL];
}

// origin 的反函式：viewBox 座標 → 棋格（拖曳落子用）
function unproject(x: number, y: number, my: WColor): Cell | null {
    const i = Math.floor((x - MARGIN) / CELL);
    const j = Math.floor((y - MARGIN) / CELL);
    const col = my === 'white' ? i : SIZE - 1 - i;
    const row = my === 'white' ? SIZE - 1 - j : j;
    if (col < 0 || col >= SIZE || row < 0 || row >= SIZE) return null;
    return [col, row];
}

const PROMO_CHOICES: WKind[] = ['queen', 'rook', 'bishop', 'knight'];
const FILES = 'abcdefgh';

function pieceFill(c: WColor) { return c === 'white' ? '#f7f7f7' : '#2b2b2b'; }
function pieceStroke(c: WColor) { return c === 'white' ? '#333' : '#111'; }

export function WesternChessBoard({
    board, myColor, lastMove, checkSide, interactive, hints, boardLabel, onMove,
}: {
    board: WBoard;
    myColor: WColor;
    lastMove: { from: Cell; to: Cell } | null;
    checkSide: WColor | null;
    interactive: boolean;
    hints: HintsData | null;
    boardLabel: string;
    onMove: (data: { from: Cell; to: Cell; promo?: string }) => void;
}) {
    const [selected, setSelected] = useState<Cell | null>(null);
    const [promo, setPromo] = useState<{ from: Cell; to: Cell } | null>(null);
    const [drag, setDrag] = useState<{ from: Cell; x: number; y: number } | null>(null);
    const svgRef = useRef<SVGSVGElement>(null);

    const promoRow = myColor === 'white' ? 7 : 0;

    const targets: Cell[] = selected
        ? (hints?.moves?.[key(selected[0], selected[1])] as Cell[] | undefined) ?? []
        : [];

    // 落子（含升變分流）
    const commit = (from: Cell, to: Cell) => {
        if (from[0] === to[0] && from[1] === to[1]) return;
        const moving = board.get(key(from[0], from[1]));
        if (moving?.kind === 'pawn' && to[1] === promoRow) {
            setPromo({ from, to }); // 等選升變子
            setSelected(null);
            return;
        }
        onMove({ from, to });
        setSelected(null);
    };

    const activate = (c: number, r: number) => {
        if (!interactive || promo) return;
        const piece = board.get(key(c, r));
        if (piece && piece.color === myColor) { setSelected([c, r]); return; }
        if (selected) commit(selected, [c, r]);
    };

    const onDown = (c: number, r: number) => (e: React.PointerEvent) => {
        sound.warmup();
        if (!interactive || promo) return;
        const piece = board.get(key(c, r));
        if (piece && piece.color === myColor) {
            setSelected([c, r]);
            if (!isTouchPointer(e)) {
                const [x, y] = origin(c, r, myColor);
                setDrag({ from: [c, r], x: x + CELL / 2, y: y + CELL / 2 });
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
    const dragging = drag ? board.get(key(drag.from[0], drag.from[1])) : undefined;

    const { cellProps } = useBoardCursor({
        cols: SIZE,
        rows: SIZE,
        enabled: interactive && !promo,
        flipped: myColor === 'black',
        onActivate: activate,
        ariaLabel: (c, r) => {
            const p = board.get(key(c, r));
            return `${FILES[c]}${r + 1}${p ? ` ${glyph(p.kind)}` : ''}`;
        },
    });

    const pickPromo = (k: WKind) => {
        if (!promo) return;
        const code = k === 'queen' ? 'q' : k === 'rook' ? 'r' : k === 'bishop' ? 'b' : 'n';
        onMove({ from: promo.from, to: promo.to, promo: code });
        setPromo(null);
    };

    const isLast = (c: number, r: number) =>
        !!lastMove && ((lastMove.from[0] === c && lastMove.from[1] === r) || (lastMove.to[0] === c && lastMove.to[1] === r));

    return (
        <div className="relative max-h-full max-w-full">
            <svg ref={svgRef} viewBox={`0 0 ${W} ${H}`} width={W} height={H}
                onPointerMove={onSvgMove} onPointerUp={onSvgUp} onPointerLeave={() => setDrag(null)}
                className="max-h-full max-w-full touch-manipulation select-none rounded-lg shadow-sm"
                role="group" aria-label={boardLabel}>
                {/* 棋格（棋盤色固定，明暗格） */}
                {Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const [x, y] = origin(c, r, myColor);
                    const dark = (c + r) % 2 === 0;
                    const sel = !!selected && selected[0] === c && selected[1] === r;
                    const last = isLast(c, r);
                    const over = !!dragOver && dragOver[0] === c && dragOver[1] === r;
                    return (
                        <g key={`sq${idx}`}>
                            <rect x={x} y={y} width={CELL} height={CELL} fill={dark ? '#b58863' : '#f0d9b5'} />
                            {last && <rect x={x} y={y} width={CELL} height={CELL} className="fill-primary-400/35" />}
                            {sel && <rect x={x} y={y} width={CELL} height={CELL} className="fill-primary-500/40" />}
                            {over && <rect x={x + 1} y={y + 1} width={CELL - 2} height={CELL - 2}
                                className="fill-none stroke-primary-500/80" strokeWidth={3} />}
                        </g>
                    );
                })}

                {/* 合法步提示（server 給的） */}
                {targets.map(([c, r], i) => {
                    const [x, y] = origin(c, r, myColor);
                    const occupied = board.has(key(c, r));
                    return occupied
                        ? <rect key={`ht${i}`} x={x + 2} y={y + 2} width={CELL - 4} height={CELL - 4}
                            className="fill-none stroke-emerald-500/80" strokeWidth={3} />
                        : <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={9} className="fill-emerald-500/45" />;
                })}

                {/* 棋子 */}
                {Array.from(board.entries()).map(([k, piece]) => {
                    const [c, r] = k.split(',').map(Number);
                    const [x, y] = origin(c, r, myColor);
                    const inCheck = checkSide === piece.color && piece.kind === 'king';
                    const isDragSrc = !!drag && drag.from[0] === c && drag.from[1] === r;
                    return (
                        <g key={k} className={inCheck ? 'animate-pulse' : undefined} opacity={isDragSrc ? 0.35 : 1}>
                            {inCheck && <rect x={x} y={y} width={CELL} height={CELL} className="fill-red-500/40" />}
                            <text x={x + CELL / 2} y={y + CELL / 2 + 2} textAnchor="middle" dominantBaseline="central"
                                fill={pieceFill(piece.color)} stroke={pieceStroke(piece.color)} strokeWidth={1}
                                style={{ fontSize: 46 }}>
                                {glyph(piece.kind)}
                            </text>
                        </g>
                    );
                })}

                {/* 跟著指標走的拖曳子 */}
                {drag && dragging && (
                    <text pointerEvents="none" x={drag.x} y={drag.y} textAnchor="middle" dominantBaseline="central"
                        fill={pieceFill(dragging.color)} stroke={pieceStroke(dragging.color)} strokeWidth={1}
                        style={{ fontSize: 46 }}>
                        {glyph(dragging.kind)}
                    </text>
                )}

                {/* 命中層：鍵盤可聚焦 + 指標按下 */}
                {!promo && Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const [x, y] = origin(c, r, myColor);
                    return <rect key={`hit${idx}`} x={x} y={y} width={CELL} height={CELL}
                        fill="transparent" className={interactive ? 'cursor-pointer focus:outline-2 focus:outline-primary-500' : ''}
                        onPointerDown={onDown(c, r)} {...cellProps(c, r)} />;
                })}
            </svg>

            {/* 升變選子 */}
            {promo && (
                <div className="absolute inset-0 flex items-center justify-center bg-neutral-900/60">
                    <div className="flex gap-2 rounded-lg bg-white p-3 shadow-lg dark:bg-neutral-800">
                        {PROMO_CHOICES.map(k => (
                            <button key={k} onClick={() => pickPromo(k)}
                                className="flex h-14 w-14 items-center justify-center rounded-md border border-neutral-300 text-4xl transition-colors hover:bg-primary-50 dark:border-neutral-600 dark:hover:bg-primary-950"
                                style={{ color: pieceFill(myColor), WebkitTextStroke: `1px ${pieceStroke(myColor)}` }}>
                                {glyph(k)}
                            </button>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
