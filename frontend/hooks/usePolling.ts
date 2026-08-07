"use client";

import { useEffect, useRef } from "react";

/**
 * 分頁在前景時才發請求的輪詢。
 *
 * 後台好幾頁是「開著就一直輪詢」的（連線列表 7s、系統指標／到訪統計 60s…），
 * 但管理員通常把分頁擱在背景不關。瀏覽器只會把背景分頁的 timer 節流到 ≥1 分鐘，
 * **不會**停掉請求本身，所以一個沒人在看的分頁仍持續打後端（1 核 1G 的機器上不划算）。
 *
 * 行為：
 * - `document.hidden` 時該次 tick 直接跳過，不發請求
 * - 回到前景時，若距離上次實際執行已超過一個週期就立刻補一次
 *   （不必再等滿一輪，切回來看到的就是新資料）
 *
 * `callback` 每次 render 取最新（存 ref），所以呼叫端不必 memo；
 * 只有 `intervalMs` / `enabled` 變動才會重排 timer。
 */
export default function usePolling(callback: () => void, intervalMs: number, enabled = true) {
    const callbackRef = useRef(callback);
    useEffect(() => {
        callbackRef.current = callback;
    });

    useEffect(() => {
        if (!enabled || intervalMs <= 0) return;

        let lastRun = Date.now();
        const run = () => {
            lastRun = Date.now();
            callbackRef.current();
        };

        const id = setInterval(() => {
            if (!document.hidden) run();
        }, intervalMs);

        const onVisibilityChange = () => {
            if (!document.hidden && Date.now() - lastRun >= intervalMs) run();
        };
        document.addEventListener("visibilitychange", onVisibilityChange);

        return () => {
            clearInterval(id);
            document.removeEventListener("visibilitychange", onVisibilityChange);
        };
    }, [enabled, intervalMs]);
}
