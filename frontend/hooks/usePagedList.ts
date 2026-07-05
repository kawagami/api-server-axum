"use client";

import { useState, useRef, useCallback, useTransition } from "react";

type Fetcher<T> = (page: number) => Promise<T[]>;

/**
 * page/per_page 式「載入更多」清單。
 * `load(fetcher)` 重設並抓第 1 頁，之後 `loadMore()` 沿用同一 fetcher 抓下一頁。
 * `initial` 可用 server 端抓好的第 1 頁 seed（免掛載後重抓）；
 * 其 fetcher 供 seed 狀態下的 `loadMore()` 用，之後任何 `load(fetcher)` 都會換掉它。
 */
export default function usePagedList<T>(perPage: number, initial?: { items: T[]; fetcher: Fetcher<T> }) {
    const [items, setItems] = useState<T[]>(initial ? initial.items : []);
    const [hasMore, setHasMore] = useState(initial ? initial.items.length >= perPage : false);
    const [isPending, startTransition] = useTransition();
    const pageRef = useRef(1);
    const fetcherRef = useRef<Fetcher<T> | null>(initial ? initial.fetcher : null);

    const load = useCallback((fetcher: Fetcher<T>) => {
        fetcherRef.current = fetcher;
        startTransition(async () => {
            try {
                const data = await fetcher(1);
                pageRef.current = 1;
                setItems(data);
                setHasMore(data.length >= perPage);
            } catch { /* adminRequest handles auth redirect */ }
        });
    }, [perPage]);

    const loadMore = useCallback(() => {
        const fetcher = fetcherRef.current;
        if (!fetcher) return;
        startTransition(async () => {
            try {
                const nextPage = pageRef.current + 1;
                const data = await fetcher(nextPage);
                pageRef.current = nextPage;
                setItems(prev => [...prev, ...data]);
                setHasMore(data.length >= perPage);
            } catch { /* adminRequest handles auth redirect */ }
        });
    }, [perPage]);

    return { items, setItems, hasMore, isPending, load, loadMore };
}
