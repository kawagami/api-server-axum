// 暗棋（翻棋）前端純資料模型。規則（位階/吃子/炮/翻子）全在 server，前端只渲染 + 套用 move_made。
// 棋盤 8 欄 × 4 列，座標 [col,row]，col 0–7、row 0–3。32 子面朝下，翻開才揭示。
// 座位 first/second（match_found.color）；紅黑由首次翻子揭示（piece.color）。

export type BColor = 'red' | 'black';
export type BKind = 'king' | 'guard' | 'elephant' | 'rook' | 'horse' | 'cannon' | 'pawn';

// hidden=true 時 color/kind 未揭示（缺）；翻開後 hidden=false 且帶 color/kind
export interface BCell { hidden: boolean; color?: BColor; kind?: BKind; }

export type Cell = [number, number];
export type BBoard = Map<string, BCell>; // "col,row" → 格內棋子

export const COLS = 8;
export const ROWS = 4;

export function key(col: number, row: number): string {
    return `${col},${row}`;
}

// 開局：32 格全部面朝下
export function initialBoard(): BBoard {
    const b: BBoard = new Map();
    for (let r = 0; r < ROWS; r++) {
        for (let c = 0; c < COLS; c++) b.set(key(c, r), { hidden: true });
    }
    return b;
}

export function applyFlip(board: BBoard, at: Cell, color: BColor, kind: BKind): BBoard {
    const next = new Map(board);
    next.set(key(at[0], at[1]), { hidden: false, color, kind });
    return next;
}

// 走/吃：from 的子移到 to（覆寫＝吃子），from 清空
export function applyMove(board: BBoard, from: Cell, to: Cell): BBoard {
    const next = new Map(board);
    const moving = next.get(key(from[0], from[1]));
    next.delete(key(from[0], from[1]));
    if (moving) next.set(key(to[0], to[1]), moving);
    return next;
}

const CHAR: Record<BColor, Record<BKind, string>> = {
    red: { king: '帥', guard: '仕', elephant: '相', rook: '俥', horse: '傌', cannon: '炮', pawn: '兵' },
    black: { king: '將', guard: '士', elephant: '象', rook: '車', horse: '馬', cannon: '砲', pawn: '卒' },
};

export function kindChar(color: BColor, kind: BKind): string {
    return CHAR[color][kind];
}
