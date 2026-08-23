"use client";

import { useCallback, useRef, useState } from 'react';

// 盤面鍵盤操作（五個盤面共用）。
//
// 為什麼需要：盤面是 SVG，命中層原本是一堆 `<circle onClick>` / `<rect onClick>` ——
// 沒有 tabIndex、沒有 role、沒有 aria-label，鍵盤與螢幕閱讀器完全無法操作，
// 整個對戰功能對不用滑鼠的人是不存在的。
//
// 用 roving tabindex（WAI-ARIA grid 慣例）：整個盤面只有一個 tab 停留點（游標所在格），
// 方向鍵移動游標、Enter/Space 落子。90 或 361 個 tab 停留點才是更糟的無障礙。
export interface BoardCursorOptions {
    cols: number;
    rows: number;
    /// 可操作時才給 tabIndex；不可操作（非我方回合 / 觀看結果）仍保留 aria-label 供閱讀
    enabled: boolean;
    /// 盤面是否上下左右翻轉顯示（象棋黑方 / 西洋棋黑方）。方向鍵走的是**視覺方向**
    flipped?: boolean;
    onActivate: (col: number, row: number) => void;
    ariaLabel: (col: number, row: number) => string;
}

const k = (c: number, r: number) => `${c},${r}`;

export function useBoardCursor({
    cols, rows, enabled, flipped = false, onActivate, ariaLabel,
}: BoardCursorOptions) {
    const [cursor, setCursor] = useState<[number, number] | null>(null);
    const nodes = useRef(new Map<string, SVGElement>());

    const focusCell = useCallback((c: number, r: number) => {
        setCursor([c, r]);
        nodes.current.get(k(c, r))?.focus();
    }, []);

    const cellProps = useCallback((c: number, r: number) => {
        const isCursor = cursor ? cursor[0] === c && cursor[1] === r : c === 0 && r === 0;
        return {
            ref: (el: SVGElement | null) => {
                if (el) nodes.current.set(k(c, r), el);
                else nodes.current.delete(k(c, r));
            },
            tabIndex: enabled ? (isCursor ? 0 : -1) : -1,
            role: 'button',
            'aria-label': ariaLabel(c, r),
            onFocus: () => setCursor([c, r]),
            onKeyDown: (e: React.KeyboardEvent) => {
                const step = flipped ? -1 : 1;
                let nc = c;
                let nr = r;
                switch (e.key) {
                    case 'ArrowRight': nc = c + step; break;
                    case 'ArrowLeft': nc = c - step; break;
                    case 'ArrowUp': nr = r + step; break;
                    case 'ArrowDown': nr = r - step; break;
                    case 'Enter':
                    case ' ':
                        e.preventDefault();
                        onActivate(c, r);
                        return;
                    default:
                        return;
                }
                e.preventDefault();
                if (nc < 0 || nc >= cols || nr < 0 || nr >= rows) return;
                focusCell(nc, nr);
            },
        };
    }, [cursor, enabled, flipped, cols, rows, onActivate, ariaLabel, focusCell]);

    return { cursor, cellProps };
}
