"use client";

import { useId, useRef, useState } from 'react';
import { sound } from '../_shared/sound';
import { useBoardCursor } from '../_shared/useBoardCursor';
import { isTouchPointer, toViewBox } from '../_shared/pointer';
import { BOARD_INK, WoodDefs, WoodSurface } from '../_shared/BoardWood';
import { useRemovedPieces } from '../_shared/useRemovedPieces';
import { XiangqiPiece, XiangqiPieceBack, XiangqiPieceDefs } from '../_shared/XiangqiPiece';
import type { HintsData } from '../_shared/wire';
import { COLS, ROWS, key, kindChar, type BBoard, type BCell, type BColor, type Cell } from './banqi-logic';

const CELL = 74;
const MARGIN = 12;
const W = COLS * CELL + 2 * MARGIN;
const H = ROWS * CELL + 2 * MARGIN;
const R = 31; // 棋子半徑

const moveId = (from: Cell, to: Cell) => `${from.join(',')}>${to.join(',')}`;

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
    // 拖曳放下的那一步：子已經在目標格了，不再播「從起點滑過去」的動畫
    const [dropped, setDropped] = useState<string | null>(null);
    const uid = useId();
    const id = (name: string) => `${uid}-${name}`;
    // 被吃淡出；翻子時舊的「背面」也會被當成移除，剛好在翻開的子底下淡掉
    const ghosts = useRemovedPieces(board, (a, b) => a.hidden === b.hidden && a.color === b.color && a.kind === b.kind);

    // 選到子時 server 給的合法目標。flips 不另外標示 —— 所有蓋著的格都能翻，畫了只是噪音
    const targets: Cell[] = selected
        ? (hints?.moves?.[key(selected[0], selected[1])] as Cell[] | undefined) ?? []
        : [];

    const commitMove = (from: Cell, to: Cell, viaDrag = false) => {
        if (from[0] === to[0] && from[1] === to[1]) return;
        setDropped(viaDrag ? moveId(from, to) : null);
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
        if (to) commitMove(drag.from, to, true);
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

    const renderCell = (cx: number, cy: number, cell: BCell, lift = 0) => cell.hidden || !cell.color || !cell.kind
        ? <XiangqiPieceBack id={id} x={cx} y={cy} r={R} />
        : <XiangqiPiece id={id} x={cx} y={cy} r={R} side={cell.color} char={kindChar(cell.color, cell.kind)} lift={lift} />;

    // 最後一手：翻子時 lastCells = [at]，走子時 = [from, to]
    const flipped = lastCells.length === 1 ? lastCells[0] : null;
    const moved = lastCells.length === 2 ? { from: lastCells[0], to: lastCells[1] } : null;

    return (
        <svg ref={svgRef} viewBox={`0 0 ${W} ${H}`} width={W} height={H}
            onPointerMove={onSvgMove} onPointerUp={onSvgUp} onPointerLeave={() => setDrag(null)}
            className="max-h-full max-w-full touch-manipulation select-none rounded-lg shadow-lg shadow-amber-950/30"
            role="group" aria-label={boardLabel}>
            <defs>
                <WoodDefs id={id} />
                <XiangqiPieceDefs id={id} />
            </defs>

            <WoodSurface id={id} w={W} h={H} />

            {/* 格線 */}
            <g stroke={BOARD_INK} strokeOpacity={0.75} strokeWidth={1.2} fill="none">
                <rect x={MARGIN} y={MARGIN} width={COLS * CELL} height={ROWS * CELL} strokeWidth={2} />
                {Array.from({ length: COLS - 1 }, (_, i) => (
                    <line key={`v${i}`} x1={MARGIN + (i + 1) * CELL} y1={MARGIN} x2={MARGIN + (i + 1) * CELL} y2={MARGIN + ROWS * CELL} />
                ))}
                {Array.from({ length: ROWS - 1 }, (_, i) => (
                    <line key={`h${i}`} x1={MARGIN} y1={MARGIN + (i + 1) * CELL} x2={MARGIN + COLS * CELL} y2={MARGIN + (i + 1) * CELL} />
                ))}
            </g>

            {/* 拖曳落點 */}
            {dragOver && (() => {
                const [x, y] = cellXY(dragOver[0], dragOver[1]);
                return <rect x={x + 1.5} y={y + 1.5} width={CELL - 3} height={CELL - 3}
                    className="fill-none stroke-primary-500/80" strokeWidth={3} />;
            })()}

            {/* 合法步提示（server 給的） */}
            {targets.map(([c, r], i) => {
                const [x, y] = cellXY(c, r);
                const occupied = board.has(key(c, r));
                return occupied
                    ? <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={R + 3}
                        className="fill-none stroke-emerald-500/80" strokeWidth={3} />
                    : <circle key={`ht${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={10} className="fill-emerald-500/45" />;
            })}

            {ghosts.map(([k, cell]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = cellXY(c, r);
                return <g key={`gh${k}`} className="piece-capture" pointerEvents="none">{renderCell(x + CELL / 2, y + CELL / 2, cell)}</g>;
            })}

            {/* 棋子 */}
            {Array.from(board.entries()).map(([k, cell]) => {
                const [c, r] = k.split(',').map(Number);
                const [x, y] = cellXY(c, r);
                const cx = x + CELL / 2;
                const cy = y + CELL / 2;
                if (cell.hidden) {
                    const pendingFlip = !!confirmFlip && confirmFlip[0] === c && confirmFlip[1] === r;
                    return (
                        <g key={`${k}:h`}>
                            {renderCell(cx, cy, cell)}
                            {/* 觸控待確認的翻子 */}
                            {pendingFlip && <circle cx={cx} cy={cy} r={R + 4} className="fill-none stroke-amber-400" strokeWidth={3} />}
                        </g>
                    );
                }
                const sel = isSel(c, r);
                const last = isLast(c, r);
                const isDragSrc = !!drag && drag.from[0] === c && drag.from[1] === r;
                // 剛翻開的子播翻面；剛走到的子從起點滑過來（拖曳放下的不滑）。
                // key 帶 color：翻開（h → 有色）與被回吃（換色）都會換元素，動畫才會重播
                const flip = !!flipped && flipped[0] === c && flipped[1] === r;
                const slide = !!moved && moved.to[0] === c && moved.to[1] === r && dropped !== moveId(moved.from, moved.to);
                let slideStyle: React.CSSProperties | undefined;
                if (slide) {
                    const [fx, fy] = cellXY(moved.from[0], moved.from[1]);
                    slideStyle = { '--dx': `${fx - x}px`, '--dy': `${fy - y}px` } as React.CSSProperties;
                }
                const lift = sel && !isDragSrc ? 3 : 0;
                return (
                    <g key={`${k}:${cell.color}`} className={flip ? 'piece-flip' : slide ? 'piece-slide' : undefined}
                        style={slideStyle} opacity={isDragSrc ? 0.35 : 1}>
                        {renderCell(cx, cy, cell, lift)}
                        {sel && <circle cx={cx} cy={cy - lift} r={R + 1.5} fill="none" strokeWidth={3} className="stroke-primary-500" />}
                        {last && !sel && <circle cx={cx} cy={cy} r={R + 4} fill="none" strokeWidth={2.5}
                            strokeDasharray="4 3" className="stroke-primary-400/80" />}
                    </g>
                );
            })}

            {/* lastMove 空格標記（移動後 from 變空格也標一下） */}
            {lastCells.map(([c, r], i) => {
                if (board.has(key(c, r))) return null;
                const [x, y] = cellXY(c, r);
                return <circle key={`le${i}`} cx={x + CELL / 2} cy={y + CELL / 2} r={R + 4} fill="none" strokeWidth={2.5}
                    strokeDasharray="4 3" className="stroke-primary-400/80" />;
            })}

            {/* 跟著指標走的拖曳子 */}
            {drag && dragging && !dragging.hidden && (
                <g pointerEvents="none">{renderCell(drag.x, drag.y, dragging, 6)}</g>
            )}

            {/* 命中層（全 32 格）：鍵盤可聚焦 + 指標按下 */}
            {Array.from({ length: COLS * ROWS }, (_, idx) => {
                const c = idx % COLS;
                const r = Math.floor(idx / COLS);
                const [x, y] = cellXY(c, r);
                return <rect key={`hit${idx}`} x={x} y={y} width={CELL} height={CELL}
                    fill="transparent" className={interactive ? 'cursor-pointer' : ''}
                    onPointerDown={onDown(c, r)} {...cellProps(c, r)} />;
            })}
        </svg>
    );
}
