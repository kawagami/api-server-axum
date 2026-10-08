import { useState } from 'react';

/// 盤面換手時，回傳「上一盤有、這一盤沒有或換了別的子」的格子，給盤面畫被吃淡出的殘影。
/// 用 render 中調整 state 的寫法（不用 ref/effect）；下一手整份換掉，不必等 animationend 清理。
export function useRemovedPieces<V>(board: Map<string, V>, same: (a: V, b: V) => boolean): [string, V][] {
    const [prev, setPrev] = useState(board);
    const [removed, setRemoved] = useState<[string, V][]>([]);
    if (board !== prev) {
        setPrev(board);
        setRemoved(Array.from(prev.entries()).filter(([k, v]) => {
            const now = board.get(k);
            return now === undefined || !same(now, v);
        }));
    }
    return removed;
}
