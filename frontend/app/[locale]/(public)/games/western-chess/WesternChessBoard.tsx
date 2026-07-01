"use client";

import { useState } from 'react';
import { sound } from '../_shared/sound';
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

const PROMO_CHOICES: WKind[] = ['queen', 'rook', 'bishop', 'knight'];

function pieceFill(c: WColor) { return c === 'white' ? '#f7f7f7' : '#2b2b2b'; }
function pieceStroke(c: WColor) { return c === 'white' ? '#333' : '#111'; }

export function WesternChessBoard({
    board, myColor, lastMove, checkSide, interactive, onMove,
}: {
    board: WBoard;
    myColor: WColor;
    lastMove: { from: Cell; to: Cell } | null;
    checkSide: WColor | null;
    interactive: boolean;
    onMove: (data: { from: Cell; to: Cell; promo?: string }) => void;
}) {
    const [selected, setSelected] = useState<Cell | null>(null);
    const [promo, setPromo] = useState<{ from: Cell; to: Cell } | null>(null);

    const promoRow = myColor === 'white' ? 7 : 0;

    const onSquare = (c: number, r: number) => {
        sound.warmup();
        if (!interactive || promo) return;
        const piece = board.get(key(c, r));
        if (piece && piece.color === myColor) { setSelected([c, r]); return; }
        if (selected) {
            const moving = board.get(key(selected[0], selected[1]));
            if (moving?.kind === 'pawn' && r === promoRow) {
                setPromo({ from: selected, to: [c, r] }); // 等選升變子
                setSelected(null);
                return;
            }
            onMove({ from: selected, to: [c, r] });
            setSelected(null);
        }
    };

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
            <svg viewBox={`0 0 ${W} ${H}`} width={W} height={H}
                className="max-h-full max-w-full select-none rounded-lg shadow" role="img">
                {/* 棋格（棋盤色固定，明暗格） */}
                {Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const [x, y] = origin(c, r, myColor);
                    const dark = (c + r) % 2 === 0;
                    const sel = !!selected && selected[0] === c && selected[1] === r;
                    const last = isLast(c, r);
                    return (
                        <g key={`sq${idx}`}>
                            <rect x={x} y={y} width={CELL} height={CELL} fill={dark ? '#b58863' : '#f0d9b5'} />
                            {last && <rect x={x} y={y} width={CELL} height={CELL} className="fill-primary-400/35" />}
                            {sel && <rect x={x} y={y} width={CELL} height={CELL} className="fill-primary-500/40" />}
                        </g>
                    );
                })}

                {/* 棋子 */}
                {Array.from(board.entries()).map(([k, piece]) => {
                    const [c, r] = k.split(',').map(Number);
                    const [x, y] = origin(c, r, myColor);
                    const inCheck = checkSide === piece.color && piece.kind === 'king';
                    return (
                        <g key={k} className={inCheck ? 'animate-pulse' : undefined}>
                            {inCheck && <rect x={x} y={y} width={CELL} height={CELL} className="fill-red-500/40" />}
                            <text x={x + CELL / 2} y={y + CELL / 2 + 2} textAnchor="middle" dominantBaseline="central"
                                fill={pieceFill(piece.color)} stroke={pieceStroke(piece.color)} strokeWidth={1}
                                style={{ fontSize: 46 }}>
                                {glyph(piece.kind)}
                            </text>
                        </g>
                    );
                })}

                {/* 命中層 */}
                {interactive && !promo && Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const [x, y] = origin(c, r, myColor);
                    return <rect key={`hit${idx}`} x={x} y={y} width={CELL} height={CELL}
                        fill="transparent" className="cursor-pointer" onClick={() => onSquare(c, r)} />;
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
