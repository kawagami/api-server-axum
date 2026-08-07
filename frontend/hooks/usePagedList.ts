"use client";

import { useState, useRef, useCallback, useTransition } from "react";
import type { PaginatedResponse } from "@/types";

type Fetcher<T> = (page: number) => Promise<PaginatedResponse<T>>;

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
 * **fetcher 回整包 `{ data, total }`**（後端全部分頁端點的統一形狀），不要在呼叫端先解包：
 * `hasMore` 是拿 `已載入筆數 < total` 算的，精確。舊版沒有 total 可用，只能猜
 * 「這頁滿了就假設還有下一頁」，最後一頁剛好滿 per_page 時會多出一顆按不出東西的
 * 「載入更多」。`total` 也一併回傳，要顯示「共 N 筆」不必再自己抓一次。
 *
 * 失敗只回 `failed` 布林、不回訊息：fetcher 幾乎都是 Server Action，
 * production 下 Next 會把拋出的 error 抹成通用訊息（只留 digest），
 * 拿不到後端的 code / message；文案交由呼叫端出（公開頁要能 i18n）。
 *
 * 每次 load / loadMore 會拿一個遞增的請求序號，回應時序號對不上就整包丟棄：
 * 連按「搜尋」「重設」或改篩選條件時舊查詢可能後回，沒有這道閘就會蓋掉新結果
 * （fetcher 是 Server Action，無法 abort，只能在回應端裁決）。
 */
export default function usePagedList<T>(initial?: { items: T[]; total: number; fetcher: Fetcher<T> }) {
    const [items, setItems] = useState<T[]>(initial ? initial.items : []);
    const [total, setTotal] = useState(initial ? initial.total : 0);
    const [loaded, setLoaded] = useState(initial ? initial.items.length : 0);
    const [failed, setFailed] = useState(false);
    const [isPending, startTransition] = useTransition();
    const pageRef = useRef(1);
    const fetcherRef = useRef<Fetcher<T> | null>(initial ? initial.fetcher : null);
    // 最新一次請求的序號；回應時比對，不是最新的就丟棄
    const reqIdRef = useRef(0);

    const load = useCallback((fetcher: Fetcher<T>) => {
        fetcherRef.current = fetcher;
        const reqId = ++reqIdRef.current;
        startTransition(async () => {
            setFailed(false);
            try {
                const page = await fetcher(1);
                if (reqIdRef.current !== reqId) return;
                pageRef.current = 1;
                setItems(page.data);
                setLoaded(page.data.length);
                setTotal(page.total);
            } catch (e) {
                if (isNavigationError(e) || reqIdRef.current !== reqId) return;
                console.error('usePagedList: load failed', e);
                setFailed(true);
            }
        });
    }, []);

    const loadMore = useCallback(() => {
        const fetcher = fetcherRef.current;
        if (!fetcher) return;
        const reqId = ++reqIdRef.current;
        // 頁碼在發請求時就推進，連點兩次「載入更多」不會重抓同一頁
        const nextPage = pageRef.current + 1;
        pageRef.current = nextPage;
        startTransition(async () => {
            setFailed(false);
            try {
                const page = await fetcher(nextPage);
                if (reqIdRef.current !== reqId) return;
                setItems(prev => [...prev, ...page.data]);
                setLoaded(prev => prev + page.data.length);
                // total 每頁都更新：清單期間有新增/刪除時，按鈕會跟著現況收掉
                setTotal(page.total);
            } catch (e) {
                if (isNavigationError(e) || reqIdRef.current !== reqId) return;
                // 這一頁沒拿到，頁碼退回去，重試才不會跳頁
                if (pageRef.current === nextPage) pageRef.current = nextPage - 1;
                console.error('usePagedList: loadMore failed', e);
                setFailed(true);
            }
        });
    }, []);

    // 「已從 server 拉了幾筆」對比總筆數。刻意不看 items.length ——
    // 呼叫端會用 setItems 就地增刪（刪一列、插一則新留言、輪詢補新紀錄），
    // 那些不代表分頁進度，混進來會讓按鈕早消失或多出來
    const hasMore = loaded < total;

    return { items, setItems, total, hasMore, isPending, failed, load, loadMore };
}
