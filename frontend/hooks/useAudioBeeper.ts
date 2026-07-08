"use client";

import { useEffect, useRef } from 'react';

export function useAudioBeeper(
    isBeeping: boolean,
    stopBeeping: () => void,
    onReturnReset: () => void,
    isRunning: boolean,
    targetTime: number | null,
) {
    const audioRef = useRef<HTMLAudioElement>(null);

    useEffect(() => {
        if (isBeeping && audioRef.current) {
            audioRef.current.play().catch(err => console.error("播放音效失敗:", err));
        } else if (!isBeeping && audioRef.current) {
            audioRef.current.pause();
            audioRef.current.currentTime = 0;
        }

        let beepTimer: ReturnType<typeof setTimeout> | undefined;
        if (isBeeping) {
            beepTimer = setTimeout(stopBeeping, 2 * 60 * 1000);
        }

        return () => clearTimeout(beepTimer);
    }, [isBeeping, stopBeeping]);

    useEffect(() => {
        // 回到頁面就停響。visibilitychange 只涵蓋同視窗切分頁;
        // Alt-Tab 跨視窗/跨 App 切回時分頁可能全程 visible、只有 window focus 事件,兩個都要聽。
        // overdue:背景分頁 timer 被節流時,時間已到但 tick 還沒補跑、isBeeping 仍為 false,
        // 此時一併重置,避免切回頁面後才開始響且再也沒有事件能停它。
        const handleReturn = () => {
            if (document.visibilityState !== 'visible') return;
            const overdue = isRunning && targetTime !== null && targetTime - Date.now() <= 0;
            if (isBeeping || overdue) {
                onReturnReset();
            }
        };
        document.addEventListener('visibilitychange', handleReturn);
        window.addEventListener('focus', handleReturn);
        return () => {
            document.removeEventListener('visibilitychange', handleReturn);
            window.removeEventListener('focus', handleReturn);
        };
    }, [isBeeping, onReturnReset, isRunning, targetTime]);

    return audioRef;
}
