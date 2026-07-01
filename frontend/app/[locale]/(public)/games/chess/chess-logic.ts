// 象棋前端純資料模型（零 DOM、零 WS）。
// 規則裁判全在後端，前端只負責：初始擺位、套用 server 廣播的走步、座標/字面對照。
// 座標系：絕對座標 [col, row]，col 0–8（左→右），row 0–9（下→上），紅在下（row 0–4）。
// 見 docs/chess-multiplayer-spec.md §8。

export type Side = 'red' | 'black';
export type PieceType =
    | 'general'   // 將/帥
    | 'advisor'   // 士/仕
    | 'elephant'  // 象/相
    | 'horse'     // 馬/傌
    | 'chariot'   // 車/俥
    | 'cannon'    // 炮/砲
    | 'soldier';  // 卒/兵

export interface Piece {
    side: Side;
    type: PieceType;
}

export type Square = [number, number]; // [col, row]

// 棋盤以 "col,row" 字串為 key 存 Piece
export type Board = Map<string, Piece>;

export const COLS = 9;
export const ROWS = 10;

export function key(col: number, row: number): string {
    return `${col},${row}`;
}

export function sameSquare(a: Square, b: Square): boolean {
    return a[0] === b[0] && a[1] === b[1];
}

// 紅黑異字（道地傳統字，見 spec §7）
const CHAR: Record<Side, Record<PieceType, string>> = {
    red: { general: '帥', advisor: '仕', elephant: '相', horse: '傌', chariot: '俥', cannon: '炮', soldier: '兵' },
    black: { general: '將', advisor: '士', elephant: '象', horse: '馬', chariot: '車', cannon: '砲', soldier: '卒' },
};

export function pieceChar(p: Piece): string {
    return CHAR[p.side][p.type];
}

// 初始擺位（spec §8.2 絕對座標）
export function initialBoard(): Board {
    const b: Board = new Map();
    const place = (side: Side, type: PieceType, squares: Square[]) => {
        for (const [c, r] of squares) b.set(key(c, r), { side, type });
    };

    // 紅方（下，row 0/2/3）
    place('red', 'chariot', [[0, 0], [8, 0]]);
    place('red', 'horse', [[1, 0], [7, 0]]);
    place('red', 'elephant', [[2, 0], [6, 0]]);
    place('red', 'advisor', [[3, 0], [5, 0]]);
    place('red', 'general', [[4, 0]]);
    place('red', 'cannon', [[1, 2], [7, 2]]);
    place('red', 'soldier', [[0, 3], [2, 3], [4, 3], [6, 3], [8, 3]]);

    // 黑方（上，row 9/7/6）
    place('black', 'chariot', [[0, 9], [8, 9]]);
    place('black', 'horse', [[1, 9], [7, 9]]);
    place('black', 'elephant', [[2, 9], [6, 9]]);
    place('black', 'advisor', [[3, 9], [5, 9]]);
    place('black', 'general', [[4, 9]]);
    place('black', 'cannon', [[1, 7], [7, 7]]);
    place('black', 'soldier', [[0, 6], [2, 6], [4, 6], [6, 6], [8, 6]]);

    return b;
}

// 套用 server 廣播的合法走步（不驗證，server 已驗）。回傳新 Board（不可變）。
export function applyMove(board: Board, from: Square, to: Square): Board {
    const next = new Map(board);
    const moving = next.get(key(from[0], from[1]));
    if (!moving) return next; // 理論上不會發生（server 權威）
    next.delete(key(from[0], from[1]));
    next.set(key(to[0], to[1]), moving); // 覆寫＝吃子
    return next;
}

export function pieceAt(board: Board, col: number, row: number): Piece | undefined {
    return board.get(key(col, row));
}

export const opponent = (s: Side): Side => (s === 'red' ? 'black' : 'red');
