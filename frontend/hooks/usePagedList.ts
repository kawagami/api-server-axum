"use client";

import { useState, useRef, useCallback, useTransition } from "react";

type Fetcher<T> = (page: number) => Promise<T[]>;

/** 401 已由 adminRequest / memberRequest 導向登入頁,不算載入失敗 */
function isNavigationError(e: unknown): boolean {
    const digest = (e as { digest?: unknown } | null)?.digest;
    return typeof digest === 'string' && (digest.startsWith('NEXT_REDIRECT') || digest === 'NEXT_NOT_FOUND');
}

/**
 * page/per_page 式「載入更多」清單。
 * `load(fetcher)` 重設並抓第 1 頁，之後 `loadMore()` 沿用同一 fetcher 抓下一頁。
 * `initial` 可用 server 端抓好的第 1 頁 seed（免掛載後重抓）；
 * 其 fetcher 供 seed 狀態下的 `loadMore()` 用，之後任何 `load(fetcher)` 都會換掉它。
 *
 * 失敗只回 `failed` 布林、不回訊息：fetcher 幾乎都是 Server Action，
 * production 下 Next 會把拋出的 error 抹成通用訊息（只留 digest），
 * 拿不到後端的 code / message；文案交由呼叫端出（公開頁要能 i18n）。
 */
export default function usePagedList<T>(perPage: number, initial?: { items: T[]; fetcher: Fetcher<T> }) {
    const [items, setItems] = useState<T[]>(initial ? initial.items : []);
    const [hasMore, setHasMore] = useState(initial ? initial.items.length >= perPage : false);
    const [failed, setFailed] = useState(false);
    const [isPending, startTransition] = useTransition();
    const pageRef = useRef(1);
    const fetcherRef = useRef<Fetcher<T> | null>(initial ? initial.fetcher : null);

    const load = useCallback((fetcher: Fetcher<T>) => {
        fetcherRef.current = fetcher;
        startTransition(async () => {
            setFailed(false);
            try {
                const data = await fetcher(1);
                pageRef.current = 1;
                setItems(data);
                setHasMore(data.length >= perPage);
            } catch (e) {
                if (isNavigationError(e)) return;
                console.error('usePagedList: load failed', e);
                setFailed(true);
            }
        });
    }, [perPage]);

    const loadMore = useCallback(() => {
        const fetcher = fetcherRef.current;
        if (!fetcher) return;
        startTransition(async () => {
            setFailed(false);
            try {
                const nextPage = pageRef.current + 1;
                const data = await fetcher(nextPage);
                pageRef.current = nextPage;
                setItems(prev => [...prev, ...data]);
                setHasMore(data.length >= perPage);
            } catch (e) {
                if (isNavigationError(e)) return;
                console.error('usePagedList: loadMore failed', e);
                setFailed(true);
            }
        });
    }, [perPage]);

    return { items, setItems, hasMore, isPending, failed, load, loadMore };
}
