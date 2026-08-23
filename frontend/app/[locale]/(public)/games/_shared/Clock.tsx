"use client";

import { useEffect, useState } from 'react';

function fmt(ms: number): string {
    const total = Math.max(0, Math.ceil(ms / 1000));
    const m = Math.floor(total / 60);
    const s = total % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
}

const TICK = 250;

// server 在每次 move_made 帶權威時鐘（剩餘 ms）；前端只負責在兩步之間平滑顯示。
//
// **顯示值是算出來的，不是累減的**：`baseMs - (now - baseAt)`。
// 累減（每 250ms 減 250）在背景分頁會失準 —— 瀏覽器把背景 timer 節流到 ≥1 秒一次，
// 回到前景時少扣掉的時間就永久留在畫面上，玩家看到的剩餘時間比實際多。
// 這樣寫也不需要父層用 key 重新掛載來重設（原本的做法，換一個時鐘值就整個元件重建）。
// 判輸仍由 server 權威（timeout_watcher）。
export function Clock({
    label, dotClass, baseMs, baseAt, running,
}: {
    label: string;
    dotClass: string;   // 標示該方的小圓點配色（各遊戲傳入）
    baseMs: number;     // server 最後給的剩餘毫秒
    baseAt: number;     // 收到該值的本地時間（Date.now()）
    running: boolean;
}) {
    const [displayMs, setDisplayMs] = useState(baseMs);

    // 每次 tick 都用 wall clock 重算（不是把上次的值減 TICK）：即使 timer 被節流、
    // 甚至整段沒被觸發，回到前景的第一次重算就會是正確的剩餘時間。
    // 計算放在 effect 而非 render 期 —— render 期呼叫 Date.now() 不是純函式（lint 會擋）。
    useEffect(() => {
        const recompute = () => setDisplayMs(Math.max(0, running ? baseMs - (Date.now() - baseAt) : baseMs));
        recompute();
        if (!running) return;
        const id = setInterval(recompute, TICK);
        return () => clearInterval(id);
    }, [running, baseMs, baseAt]);

    const low = displayMs <= 30_000;

    return (
        <div
            className={[
                'flex items-center justify-between gap-3 rounded-lg border px-4 py-2 transition-colors',
                running
                    ? 'border-primary-400 bg-primary-50 dark:border-primary-600 dark:bg-primary-950'
                    : 'border-neutral-200 bg-neutral-50 dark:border-neutral-700 dark:bg-neutral-800',
            ].join(' ')}
        >
            <span className="flex items-center gap-2 text-sm font-medium text-neutral-700 dark:text-neutral-200">
                <span className={`inline-block h-3 w-3 rounded-full ${dotClass}`} />
                {label}
            </span>
            <span
                className={[
                    'font-mono text-lg tabular-nums',
                    low ? 'text-red-600 dark:text-red-400' : 'text-neutral-800 dark:text-neutral-100',
                    running && low ? 'animate-pulse' : '',
                ].join(' ')}
            >
                {fmt(displayMs)}
            </span>
        </div>
    );
}
