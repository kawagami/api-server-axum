"use client";

import { useCallback, useMemo } from "react";
import { usePathname, useSearchParams } from "next/navigation";

export type FilterValue = string | boolean;

/**
 * 把後台清單的篩選條件同步進 URL query —— 重新整理不掉條件、連結可以直接貼給別人看同一份查詢。
 *
 * 只做「讀初始值」與「寫回 URL」兩件事，資料抓取仍由呼叫端的 usePagedList 負責。
 *
 * 寫回用 `history.replaceState` 而不是 `router.replace`：
 * - 不觸發 RSC 導航（清單已在 client 端抓好了，再跑一次 server render 純屬浪費）
 * - 不動捲軸位置
 * - replace 而非 push，避免每改一個下拉就在「上一頁」堆一筆歷程
 *   （代價：上一頁不會逐步倒回先前的篩選條件，對篩選列來說是刻意取捨）
 *
 * 泛型約束寫成 mapped type 而不是 `Record<string, FilterValue>`，介面（沒有 index signature）
 * 才能直接當 T 傳進來。
 */
export default function useFilterUrl<T extends { [K in keyof T]: FilterValue }>(defaults: T) {
    const pathname = usePathname();
    const searchParams = useSearchParams();

    // 只在首次渲染讀一次；之後 URL 由 write() 單向寫出，不再回頭當資料來源（免與 state 打架）
    const initial = useMemo<T>(() => {
        const decoded = (Object.keys(defaults) as (keyof T & string)[]).map(key => {
            const fallback = defaults[key];
            const raw = searchParams.get(key);
            if (typeof fallback === "boolean") {
                return [key, raw === null ? fallback : raw === "true"];
            }
            return [key, raw ?? fallback];
        });
        return Object.fromEntries(decoded) as T;
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, []);

    /**
     * 寫回 URL。與預設值相同的 key 不寫（URL 保持乾淨、可讀）。
     * 允許傳序列化過的形狀（如把 datetime-local 轉成 ISO 再寫），只要欄位型別對得上。
     */
    const write = useCallback((values: T) => {
        const params = new URLSearchParams();
        for (const key of Object.keys(values) as (keyof T & string)[]) {
            const value = values[key];
            if (value === defaults[key]) continue;
            if (typeof value === "boolean") {
                if (value) params.set(key, "true");
            } else if (value) {
                params.set(key, value);
            }
        }
        const qs = params.toString();
        window.history.replaceState(null, "", qs ? `${pathname}?${qs}` : pathname);
        // defaults 是呼叫端的模組層常數，不進依賴
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [pathname]);

    return { initial, write };
}
