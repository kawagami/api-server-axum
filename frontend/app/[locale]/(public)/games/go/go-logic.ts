// 圍棋前端純資料模型。規則（提子/劫/自殺/數子）全在 server，前端只擺子 + 依 captured 提子。
// 19×19，棋子下在交叉點，座標 [col,row] 皆 0–18。黑先。

export type GColor = 'black' | 'white';
export type Cell = [number, number];
export type GBoard = Map<string, GColor>; // "col,row" → 子色

export const SIZE = 19;
// 星位（hoshi）
export const STARS: Cell[] = [
    [3, 3], [9, 3], [15, 3],
    [3, 9], [9, 9], [15, 9],
    [3, 15], [9, 15], [15, 15],
];

export function key(col: number, row: number): string {
    return `${col},${row}`;
}

export function emptyBoard(): GBoard {
    return new Map();
}

// 落子 + 提走 captured 列表（server 算好）
export function applyPlay(board: GBoard, at: Cell, by: GColor, captured: Cell[]): GBoard {
    const next = new Map(board);
    next.set(key(at[0], at[1]), by);
    for (const [c, r] of captured) next.delete(key(c, r));
    return next;
}
