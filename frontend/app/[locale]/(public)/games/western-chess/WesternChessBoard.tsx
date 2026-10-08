"use client";

import { useId, useRef, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer, toViewBox } from '../_shared/pointer';
import { BOARD_INK, WoodDefs, WoodSurface } from '../_shared/BoardWood';
import { useRemovedPieces } from '../_shared/useRemovedPieces';
import type { HintsData } from '../_shared/wire';
import { WcPieceIcon, WcPieceShape } from './WcPiece';
import { SIZE, glyph, key, type Cell, type WBoard, type WColor, type WKind, type Piece } from './wc-logic';

const CELL = 64;
const MARGIN = 24; // 木框寬，座標字畫在框上
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

const BOARD = SIZE * CELL;
// 格色與標示色沿用 Lichess 慣例：黃綠標最後一手、深綠標選中與合法步，在深淺格上都看得清楚
const SQ_LIGHT = '#f0d9b5';
const SQ_DARK = '#b58863';
const LAST_MOVE = 'rgba(155, 199, 0, 0.41)';
const MARK = 'rgba(20, 85, 30, 0.5)';

const moveId = (from: Cell, to: Cell) => `${from.join(',')}>${to.join(',')}`;

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
    // 拖曳放下的那一步：子已經在目標格了，不再播「從起點滑過去」的動畫
    const [dropped, setDropped] = useState<string | null>(null);
    const uid = useId();
    const id = (name: string) => `${uid}-${name}`;
    const ghosts = useRemovedPieces(board, (a, b) => a.color === b.color && a.kind === b.kind); // 被吃淡出

    const promoRow = myColor === 'white' ? 7 : 0;

    const targets: Cell[] = selected
        ? (hints?.moves?.[key(selected[0], selected[1])] as Cell[] | undefined) ?? []
        : [];

    // 落子（含升變分流）
    const commit = (from: Cell, to: Cell, viaDrag = false) => {
        if (from[0] === to[0] && from[1] === to[1]) return;
        setDropped(viaDrag ? moveId(from, to) : null);
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
        if (to) commit(drag.from, to, true);
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

    // 棋子：底部橢圓陰影 + Cburnett 圖形（45×45 縮放進一格）。lift 是浮起高度（拖曳）
    const renderPiece = (x: number, y: number, p: Piece, lift = 0) => (
        <>
            <ellipse cx={x + CELL / 2 + 1 + lift} cy={y + CELL * 0.84 + lift} rx={CELL * 0.3 + lift} ry={CELL * 0.075}
                fill={`url(#${id('shadow')})`} />
            <g transform={`translate(${x + CELL * 0.03} ${y + CELL * 0.03 - lift}) scale(${(CELL * 0.94) / 45})`}>
                <WcPieceShape kind={p.kind} color={p.color} />
            </g>
        </>
    );

    // 座標字：檔案（a–h）在下框、橫列（1–8）在左框，跟著視角翻轉
    const coords: React.ReactNode[] = [];
    for (let i = 0; i < SIZE; i++) {
        const [fx] = origin(i, 0, myColor);
        const [, ry] = origin(0, i, myColor);
        coords.push(<text key={`f${i}`} x={fx + CELL / 2} y={MARGIN + BOARD + MARGIN / 2}>{FILES[i]}</text>);
        coords.push(<text key={`r${i}`} x={MARGIN / 2} y={ry + CELL / 2}>{i + 1}</text>);
    }

    return (
        <div className="relative max-h-full max-w-full">
            <svg ref={svgRef} viewBox={`0 0 ${W} ${H}`} width={W} height={H}
                onPointerMove={onSvgMove} onPointerUp={onSvgUp} onPointerLeave={() => setDrag(null)}
                className="max-h-full max-w-full touch-manipulation select-none rounded-lg shadow-lg shadow-amber-950/30"
                role="group" aria-label={boardLabel}>
                <defs>
                    <WoodDefs id={id} />
                    {/* 被將軍：王底下的紅色光暈（Lichess 同款），比整格塗紅不刺眼 */}
                    <radialGradient id={id('check')}>
                        <stop offset="0" stopColor="#ff0000" />
                        <stop offset="0.25" stopColor="#e70000" />
                        <stop offset="0.9" stopColor="#a90000" stopOpacity="0" />
                    </radialGradient>
                </defs>

                {/* 木框（整張底）→ 棋格 → 棋格上再疊一層木紋 → 深色模式壓暗 */}
                <WoodSurface id={id} w={W} h={H} />
                {Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const [x, y] = origin(c, r, myColor);
                    return <rect key={`sq${idx}`} x={x} y={y} width={CELL} height={CELL}
                        fill={(c + r) % 2 === 0 ? SQ_DARK : SQ_LIGHT} />;
                })}
                <rect x={MARGIN} y={MARGIN} width={BOARD} height={BOARD} filter={`url(#${id('grain')})`} opacity={0.3} />
                <rect x={MARGIN} y={MARGIN} width={BOARD} height={BOARD} fill="#2a1405" className="opacity-0 dark:opacity-[0.22]" />
                <rect x={MARGIN} y={MARGIN} width={BOARD} height={BOARD} fill="none" stroke={BOARD_INK} strokeOpacity={0.7} strokeWidth={1.5} />

                <g fill={BOARD_INK} fillOpacity={0.8} textAnchor="middle" dominantBaseline="central"
                    style={{ fontSize: 13, fontWeight: 600 }}>
                    {coords}
                </g>

                {/* 最後一手／選中／拖曳落點 */}
                {Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const sel = !!selected && selected[0] === c && selected[1] === r;
                    const last = isLast(c, r);
                    const over = !!dragOver && dragOver[0] === c && dragOver[1] === r;
                    if (!sel && !last && !over) return null;
                    const [x, y] = origin(c, r, myColor);
                    return (
                        <g key={`mk${idx}`}>
                            {last && <rect x={x} y={y} width={CELL} height={CELL} fill={LAST_MOVE} />}
                            {sel && <rect x={x} y={y} width={CELL} height={CELL} fill={MARK} />}
                            {over && <rect x={x + 1.5} y={y + 1.5} width={CELL - 3} height={CELL - 3}
                                fill="none" stroke="rgba(255, 255, 255, 0.75)" strokeWidth={3} />}
                        </g>
                    );
                })}

                {/* 被將軍的王底下光暈 */}
                {Array.from(board.entries()).map(([k, piece]) => {
                    if (checkSide !== piece.color || piece.kind !== 'king') return null;
                    const [c, r] = k.split(',').map(Number);
                    const [x, y] = origin(c, r, myColor);
                    return <rect key={`chk${k}`} x={x} y={y} width={CELL} height={CELL} fill={`url(#${id('check')})`} />;
                })}

                {/* 合法步提示（server 給的）：空格畫點、可吃子畫環 */}
                {targets.map(([c, r], i) => {
                    const [x, y] = origin(c, r, myColor);
                    const occupied = board.has(key(c, r));
                    return occupied
                        ? <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={CELL / 2 - 3.5}
                            fill="none" stroke={MARK} strokeWidth={6} />
                        : <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={CELL * 0.16} fill={MARK} />;
                })}

                {ghosts.map(([k, piece]) => {
                    const [c, r] = k.split(',').map(Number);
                    const [x, y] = origin(c, r, myColor);
                    return <g key={`g${k}`} className="piece-capture" pointerEvents="none">{renderPiece(x, y, piece)}</g>;
                })}

                {/* 棋子 */}
                {Array.from(board.entries()).map(([k, piece]) => {
                    const [c, r] = k.split(',').map(Number);
                    const [x, y] = origin(c, r, myColor);
                    const isDragSrc = !!drag && drag.from[0] === c && drag.from[1] === r;
                    // 最後一手的落點從起點滑過來（拖曳放下的不滑）。key 帶 color：同一格被回吃時換元素，動畫才會重播
                    const slide = !!lastMove && lastMove.to[0] === c && lastMove.to[1] === r
                        && dropped !== moveId(lastMove.from, lastMove.to);
                    let slideStyle: React.CSSProperties | undefined;
                    if (slide) {
                        const [fx, fy] = origin(lastMove.from[0], lastMove.from[1], myColor);
                        slideStyle = { '--dx': `${fx - x}px`, '--dy': `${fy - y}px` } as React.CSSProperties;
                    }
                    return (
                        <g key={`${k}:${piece.color}`} className={slide ? 'piece-slide' : undefined} style={slideStyle}
                            opacity={isDragSrc ? 0.35 : 1}>
                            {renderPiece(x, y, piece)}
                        </g>
                    );
                })}

                {/* 跟著指標走的拖曳子（稍微放大、浮起） */}
                {drag && dragging && (
                    <g pointerEvents="none">
                        {renderPiece(drag.x - CELL / 2, drag.y - CELL / 2, dragging, 4)}
                    </g>
                )}

                {/* 命中層：鍵盤可聚焦 + 指標按下 */}
                {!promo && Array.from({ length: SIZE * SIZE }, (_, idx) => {
                    const c = idx % SIZE;
                    const r = Math.floor(idx / SIZE);
                    const [x, y] = origin(c, r, myColor);
                    return <rect key={`hit${idx}`} x={x} y={y} width={CELL} height={CELL}
                        fill="transparent" className={interactive ? 'cursor-pointer' : ''}
                        onPointerDown={onDown(c, r)} {...cellProps(c, r)} />;
                })}
            </svg>

            {/* 升變選子 */}
            {promo && (
                <div className="absolute inset-0 flex items-center justify-center bg-neutral-900/60">
                    <div className="flex gap-2 rounded-lg bg-white p-3 shadow-lg dark:bg-neutral-800">
                        {PROMO_CHOICES.map(k => (
                            <button key={k} onClick={() => pickPromo(k)}
                                aria-label={glyph(k)}
                                className="flex h-14 w-14 items-center justify-center rounded-md border border-neutral-300 transition-colors hover:bg-primary-50 dark:border-neutral-600 dark:hover:bg-primary-950">
                                <WcPieceIcon kind={k} color={myColor} className="h-12 w-12" />
                            </button>
                        ))}
                    </div>
                </div>
            )}
        </div>
    );
}
