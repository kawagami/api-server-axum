"use client";

import { useCallback, useMemo, useRef, useState } from "react";

/**
 * 後台清單的多選狀態（配 `components/admin/bulk-select.tsx`）。
 *
 * `orderedIds` 是**目前畫面上的列、依顯示順序**。對外的選取結果一律與它取交集：
 * 列被刪掉或清單重載後，不在畫面上的 id 自動不算數，呼叫端不必在每個刪除點手動清。
 * 交集是在 render 時算的（不是 effect 裡修 state），所以不會多一輪 render。
 *
 * shift + 點擊做範圍選取：從上一次點的那列到這一列，全部設成「這一列切換後的狀態」
 * （跟檔案總管一樣，範圍內混雜的勾選狀態會被統一）。
 */
export default function useSelection<K extends number | string>(orderedIds: K[]) {
    const [selected, setSelected] = useState<Set<K>>(() => new Set());
    // 範圍選取的起點：上一次「單點」的那列
    const anchorRef = useRef<K | null>(null);

    const selectedIds = useMemo(
        () => orderedIds.filter(id => selected.has(id)),
        [orderedIds, selected],
    );

    const toggle = useCallback((id: K, shiftKey = false) => {
        const anchor = anchorRef.current;
        setSelected(prev => {
            const next = new Set(prev);
            const on = !prev.has(id);
            const from = anchor === null ? -1 : orderedIds.indexOf(anchor);
            const to = orderedIds.indexOf(id);
            if (shiftKey && from !== -1 && to !== -1) {
                const [lo, hi] = from < to ? [from, to] : [to, from];
                for (const k of orderedIds.slice(lo, hi + 1)) {
                    if (on) next.add(k);
                    else next.delete(k);
                }
            } else if (on) {
                next.add(id);
            } else {
                next.delete(id);
            }
            return next;
        });
        anchorRef.current = id;
    }, [orderedIds]);

    const count = selectedIds.length;
    const allSelected = orderedIds.length > 0 && count === orderedIds.length;

    /** 表頭的全選鈕：已全選就全取消，否則選滿畫面上的列（只含已載入的，不是後端全部） */
    const toggleAll = useCallback(() => {
        setSelected(allSelected ? new Set() : new Set(orderedIds));
        anchorRef.current = null;
    }, [allSelected, orderedIds]);

    const clear = useCallback(() => {
        setSelected(new Set());
        anchorRef.current = null;
    }, []);

    const isSelected = useCallback((id: K) => selected.has(id), [selected]);

    return {
        selectedIds,
        count,
        allSelected,
        /** 有選但沒全選（表頭 checkbox 的 indeterminate） */
        someSelected: count > 0 && !allSelected,
        isSelected,
        toggle,
        toggleAll,
        clear,
    };
}

export type Selection<K extends number | string> = ReturnType<typeof useSelection<K>>;
