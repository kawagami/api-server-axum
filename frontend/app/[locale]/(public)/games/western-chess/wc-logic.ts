// 西洋棋前端純資料模型。規則全在 server，前端只擺子 + 套用 move_made（含易位/過路兵/升變旗標）。
// 8×8，座標 [col,row]，col 0–7（a–h），row 0–7，白方底線 row 0。絕對座標，翻轉純渲染。

export type WColor = 'white' | 'black';
export type WKind = 'king' | 'queen' | 'rook' | 'bishop' | 'knight' | 'pawn';

export interface Piece { color: WColor; kind: WKind; }
export type Cell = [number, number];
export type WBoard = Map<string, Piece>; // "col,row" → Piece

export const SIZE = 8;

export function key(col: number, row: number): string {
    return `${col},${row}`;
}

const BACK: WKind[] = ['rook', 'knight', 'bishop', 'queen', 'king', 'bishop', 'knight', 'rook'];

export function initialBoard(): WBoard {
    const b: WBoard = new Map();
    for (let c = 0; c < SIZE; c++) {
        b.set(key(c, 0), { color: 'white', kind: BACK[c] });
        b.set(key(c, 1), { color: 'white', kind: 'pawn' });
        b.set(key(c, 6), { color: 'black', kind: 'pawn' });
        b.set(key(c, 7), { color: 'black', kind: BACK[c] });
    }
    return b;
}

// 實心字符（白子另加描邊區分，見 Board）
const GLYPH: Record<WKind, string> = {
    king: '♚', queen: '♛', rook: '♜', bishop: '♝', knight: '♞', pawn: '♟',
};
export function glyph(kind: WKind): string {
    return GLYPH[kind];
}

const PROMO: Record<string, WKind> = { q: 'queen', r: 'rook', b: 'bishop', n: 'knight' };

export interface WMoveMade {
    from: Cell; to: Cell;
    promo?: string;          // 升變 → 替換 to 的子
    castle?: 'king' | 'queen'; // 王車易位 → 連帶移動車
    ep_capture?: Cell;        // 吃過路兵 → 移除此格
}

// 套用 server 廣播的走步。回傳 { board, capture }
export function applyMove(board: WBoard, d: WMoveMade): { board: WBoard; capture: boolean } {
    const next = new Map(board);
    const fromK = key(d.from[0], d.from[1]);
    const toK = key(d.to[0], d.to[1]);
    const moving = next.get(fromK);
    const capture = next.has(toK) || !!d.ep_capture;

    next.delete(fromK);
    if (moving) {
        const placed: Piece = d.promo ? { color: moving.color, kind: PROMO[d.promo] ?? 'queen' } : moving;
        next.set(toK, placed);
    }

    if (d.castle) {
        const row = d.to[1];
        if (d.castle === 'king') { // 王翼：車 h→f
            const r = next.get(key(7, row));
            next.delete(key(7, row));
            if (r) next.set(key(5, row), r);
        } else { // 后翼：車 a→d
            const r = next.get(key(0, row));
            next.delete(key(0, row));
            if (r) next.set(key(3, row), r);
        }
    }

    if (d.ep_capture) next.delete(key(d.ep_capture[0], d.ep_capture[1]));

    return { board: next, capture };
}
