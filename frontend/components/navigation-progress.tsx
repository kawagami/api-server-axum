"use client";

import { Suspense, useEffect, useRef, useState } from "react";
// 前後台共用、只拿來偵測「網址變了」：要的是含 locale 的原始 pathname，不用 i18n 版
// eslint-disable-next-line no-restricted-imports
import { usePathname, useSearchParams } from "next/navigation";

/**
 * 全站換頁進度條（掛在 root layout，前後台共用）。
 *
 * 為什麼需要：`loading.tsx` 要等 prefetch 抓到才會「點了立刻出骨架」，慢網路下 prefetch 還沒回來，
 * 點下去畫面完全不動，直到 server 回應才切換 —— 使用者分不出是沒點到還是在跑。
 *
 * 做法：document 層攔站內 `<a>` 的點擊開始跑，pathname / searchParams 變了就收尾。
 * - `useLinkStatus` 只能用在個別 `<Link>` 底下，做不到全域，所以不用它
 * - 只吃連結點擊；`router.push` 的呼叫端（分頁、搜尋、篩選、status-link）各自已有 `isPending` 回饋
 * - 前 150ms 不顯示：prefetch 命中的換頁幾乎瞬間完成，不該閃一下
 * - 20 秒保底收掉：避免換頁失敗 / 連結被別的 handler 攔下時永遠卡著
 */
export default function NavigationProgress() {
    // useSearchParams 必須包 Suspense，否則整個 root layout 會退成 client render
    return (
        <Suspense fallback={null}>
            <Bar />
        </Suspense>
    );
}

const SHOW_DELAY_MS = 150;
const TRICKLE_MS = 300;
const GIVE_UP_MS = 20_000;

function Bar() {
    const pathname = usePathname();
    const searchParams = useSearchParams();
    const routeKey = `${pathname}?${searchParams.toString()}`;

    // progress：null = 閒置；0~1 = 進行中；visible 控制 150ms 延遲顯示
    const [progress, setProgress] = useState<number | null>(null);
    const [visible, setVisible] = useState(false);
    const timers = useRef<number[]>([]);
    const active = useRef(false);

    function clearTimers() {
        timers.current.forEach(id => { clearTimeout(id); clearInterval(id); });
        timers.current = [];
    }

    function finish() {
        if (!active.current) return;
        active.current = false;
        clearTimers();
        setProgress(1);
        // 跑滿後淡出，再歸零（淡出期間仍顯示在 100%）
        timers.current.push(window.setTimeout(() => setVisible(false), 200));
        timers.current.push(window.setTimeout(() => setProgress(null), 500));
    }

    function start() {
        if (active.current) return;
        active.current = true;
        clearTimers();
        setVisible(false);
        setProgress(0.08);
        timers.current.push(window.setTimeout(() => setVisible(true), SHOW_DELAY_MS));
        // 逼近 90% 但不到頂：越接近越慢，給人「還在動」的感覺
        timers.current.push(window.setInterval(() => setProgress(p => (p === null ? p : p + (0.9 - p) * 0.08)), TRICKLE_MS));
        timers.current.push(window.setTimeout(finish, GIVE_UP_MS));
    }

    // finish / start 只碰 ref 與 setState，放進 ref 讓監聽器只註冊一次
    const startRef = useRef(start);
    const finishRef = useRef(finish);
    useEffect(() => {
        startRef.current = start;
        finishRef.current = finish;
    });

    useEffect(() => {
        function onClick(e: MouseEvent) {
            if (e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
            const a = (e.target as Element | null)?.closest?.("a");
            if (!a || !a.href || a.hasAttribute("download")) return;
            if (a.target && a.target !== "_self") return;
            const url = new URL(a.href, location.href);
            if (url.origin !== location.origin) return;
            // 同頁（含只差 #hash）不會換頁
            if (url.pathname === location.pathname && url.search === location.search) return;
            startRef.current();
        }
        document.addEventListener("click", onClick, true);
        return () => {
            document.removeEventListener("click", onClick, true);
            clearTimers();
        };
    }, []);

    // 換頁完成（含上一頁 / 下一頁）→ 收尾
    useEffect(() => {
        finishRef.current();
    }, [routeKey]);

    if (progress === null) return null;
    return (
        <div
            aria-hidden
            className={`pointer-events-none fixed inset-x-0 top-0 z-[70] h-0.5 transition-opacity duration-300 ${visible ? "opacity-100" : "opacity-0"}`}
        >
            <div
                className="h-full origin-left bg-primary-500 dark:bg-primary-400 transition-transform duration-300 ease-out motion-reduce:transition-none"
                style={{ transform: `scaleX(${progress})` }}
            />
        </div>
    );
}
