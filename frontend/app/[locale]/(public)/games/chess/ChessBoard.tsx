"use client";

import { useId, useRef, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer, toViewBox } from '../_shared/pointer';
import { BOARD_INK, PieceShadow, WoodDefs, WoodSurface } from '../_shared/BoardWood';
import { useRemovedPieces } from '../_shared/useRemovedPieces';
import type { HintsData } from '../_shared/wire';
import { key, pieceChar, type Board as BoardModel, type Piece, type Side, type Square } from './chess-logic';

const CELL = 64;
const MARGIN = 36;
const W = 8 * CELL + 2 * MARGIN;
const H = 9 * CELL + 2 * MARGIN;
const R = 26; // 棋子半徑
// 楷書優先，沒有就退回系統襯線字；棋子字與河界共用
const KAI_FONT = '"BiauKai", "DFKai-SB", "KaiTi", "STKaiti", "Kaiti TC", serif';
const INK = { red: '#a8231a', black: '#1f1a14' } as const;

const moveId = (from: Square, to: Square) => `${from.join(',')}>${to.join(',')}`;

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
    // 拖曳放下的那一步：子已經在目標格了，不再播「從起點滑過去」的動畫
    const [dropped, setDropped] = useState<string | null>(null);
    const uid = useId();
    const id = (name: string) => `${uid}-${name}`;
    const ghosts = useRemovedPieces(board, (a, b) => a.side === b.side && a.type === b.type); // 被吃淡出

    // 選到子時 server 給的合法目標（純提示；判定仍在後端）
    const targets: Square[] = selected
        ? (hints?.moves?.[key(selected[0], selected[1])] as Square[] | undefined) ?? []
        : [];
    const isTarget = (c: number, r: number) => targets.some(([tc, tr]) => tc === c && tr === r);

    const commit = (from: Square, to: Square, viaDrag = false) => {
        if (from[0] === to[0] && from[1] === to[1]) return;
        setDropped(viaDrag ? moveId(from, to) : null);
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
        if (to) commit(drag.from, to, true);
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

    // 河界在第 4、5 列正中間；取兩列中點，不要用「第 4 列 ± 半格」—— 翻轉視角時方向會反
    const [riverX, riverY4] = project(4, 4, myColor);
    const riverY = (riverY4 + project(4, 5, myColor)[1]) / 2;
    const dragging = board.get(drag ? key(drag.from[0], drag.from[1]) : '');

    // 木頭棋子：陰影 → 木面 → 外緣 → 內圈刻線 → 字。lift 是浮起高度（選中、拖曳）
    const renderPiece = (x: number, y: number, p: Piece, lift = 0) => (
        <>
            <PieceShadow id={id} x={x} y={y} r={R} lift={lift} />
            <g transform={lift ? `translate(0 ${-lift})` : undefined}>
                <circle cx={x} cy={y} r={R} fill={`url(#${id('piece')})`} stroke="#8a6232" strokeWidth={1.5} />
                <circle cx={x} cy={y} r={R - 4.5} fill="none" stroke={INK[p.side]} strokeOpacity={0.7} strokeWidth={1.3} />
                <text x={x} y={y + 1} textAnchor="middle" dominantBaseline="central" fill={INK[p.side]}
                    style={{ fontSize: 29, fontWeight: 700, fontFamily: KAI_FONT }}>
                    {pieceChar(p)}
                </text>
            </g>
        </>
    );

    return (
        <svg ref={svgRef} viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            onPointerMove={onSvgMove} onPointerUp={onSvgUp} onPointerLeave={() => setDrag(null)}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg shadow-lg shadow-amber-950/30"
            role="group" aria-label={boardLabel}>
            <defs>
                <WoodDefs id={id} />
                <radialGradient id={id('piece')} cx="0.38" cy="0.32" r="0.75">
                    <stop offset="0" stopColor="#fbedcc" />
                    <stop offset="0.6" stopColor="#ecd29e" />
                    <stop offset="1" stopColor="#d2ab68" />
                </radialGradient>
            </defs>

            <WoodSurface id={id} w={W} h={H} />

            <g stroke={BOARD_INK} strokeOpacity={0.75} strokeWidth={1.5} fill="none">{lines}</g>

            <text x={riverX} y={riverY} textAnchor="middle" dominantBaseline="middle"
                fill={BOARD_INK} fillOpacity={0.6} style={{ fontSize: 26, letterSpacing: 10, fontFamily: KAI_FONT }}>
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

            {ghosts.map(([k, piece]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = project(c, r, myColor);
                return <g key={`g${k}`} className="piece-capture" pointerEvents="none">{renderPiece(x, y, piece)}</g>;
            })}

            {Array.from(board.entries()).map(([k, piece]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = project(c, r, myColor);
                const isSel = !!selected && selected[0] === c && selected[1] === r;
                const inCheck = checkSide === piece.side && piece.type === 'general';
                const isDragSrc = !!drag && drag.from[0] === c && drag.from[1] === r;
                // 最後一手的落點從起點滑過來（拖曳放下的不滑）。key 帶 side：同一格被回吃時換一個元素，動畫才會重播
                const slide = !!lastMove && lastMove.to[0] === c && lastMove.to[1] === r
                    && dropped !== moveId(lastMove.from, lastMove.to);
                let slideStyle: React.CSSProperties | undefined;
                if (slide) {
                    const [fx, fy] = project(lastMove.from[0], lastMove.from[1], myColor);
                    slideStyle = { '--dx': `${fx - x}px`, '--dy': `${fy - y}px` } as React.CSSProperties;
                }
                const lift = isSel && !isDragSrc ? 3 : 0;
                return (
                    <g key={`${k}:${piece.side}`} className={slide ? 'piece-slide' : undefined} style={slideStyle}
                        opacity={isDragSrc ? 0.35 : 1}>
                        <g className={inCheck ? 'animate-pulse' : undefined}>
                            {renderPiece(x, y, piece, lift)}
                            {(isSel || inCheck) && (
                                <circle cx={x} cy={y - lift} r={R + 1.5} fill="none" strokeWidth={3}
                                    className={isSel ? 'stroke-primary-500' : 'stroke-red-500'} />
                            )}
                        </g>
                    </g>
                );
            })}

            {/* 跟著指標走的拖曳子 */}
            {drag && dragging && (
                <g pointerEvents="none">
                    {renderPiece(drag.x, drag.y, dragging, 6)}
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
