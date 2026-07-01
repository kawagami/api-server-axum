// 五子棋前端純資料模型。規則（五連勝/滿盤和）全在 server，前端只擺子。
// 座標 [col,row] 皆 0–14，黑先。

export type GColor = 'black' | 'white';
export type Cell = [number, number];
export type GBoard = Map<string, GColor>; // "col,row" → 落子方

export const SIZE = 15;

export function key(col: number, row: number): string {
    return `${col},${row}`;
}

export function emptyBoard(): GBoard {
    return new Map();
}

export function placeStone(board: GBoard, at: Cell, by: GColor): GBoard {
    const next = new Map(board);
    next.set(key(at[0], at[1]), by);
    return next;
}
