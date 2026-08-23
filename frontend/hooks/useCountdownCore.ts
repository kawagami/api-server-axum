"use client";

import { useState, useEffect, useCallback } from 'react';
import { startSecondTick } from '@/libs/second-tick';

/**
 * 鬧鐘（useAlarm）與倒數計時（useTimer）共用的核心狀態機：
 * 對齊秒邊界的 tick、剩餘秒數、到點轉響鈴、暫停／清除。
 * 兩者真正的差異（目標時刻怎麼算、有沒有暫停、清除後顯示什麼）留給呼叫端。
 * initialTimeLeft = 尚未開始時顯示的秒數（鬧鐘 0、倒數是預設分鐘數換算）。
 */
export default function useCountdownCore(initialTimeLeft = 0) {
    const [targetTime, setTargetTime] = useState<number | null>(null);
    const [timeLeft, setTimeLeft] = useState(initialTimeLeft);
    const [isRunning, setIsRunning] = useState(false);
    const [isBeeping, setIsBeeping] = useState(false);

    useEffect(() => {
        if (!isRunning || targetTime === null) return;
        return startSecondTick(() => {
            const remaining = Math.max(0, Math.floor((targetTime - Date.now()) / 1000));
            setTimeLeft(remaining);
            if (remaining === 0) {
                setIsRunning(false);
                setIsBeeping(true);
                setTargetTime(null);
            }
        }, targetTime);
    }, [isRunning, targetTime]);

    /** 設定目標時刻（epoch ms）並開始跑 */
    const startAt = useCallback((target: number) => {
        setTargetTime(target);
        setTimeLeft(Math.max(0, Math.floor((target - Date.now()) / 1000)));
        setIsRunning(true);
        setIsBeeping(false);
    }, []);

    /** 暫停：保留 targetTime，呼叫端用 `!isRunning && targetTime !== null` 判斷暫停中 */
    const pause = useCallback(() => setIsRunning(false), []);

    /** 停止並回到閒置；idleTimeLeft = 閒置時要顯示的秒數（鬧鐘 0、倒數是設定值） */
    const clear = useCallback((idleTimeLeft: number) => {
        setIsRunning(false);
        setIsBeeping(false);
        setTargetTime(null);
        setTimeLeft(idleTimeLeft);
    }, []);

    const stopBeeping = useCallback(() => setIsBeeping(false), []);

    return { targetTime, timeLeft, isRunning, isBeeping, startAt, pause, clear, stopBeeping };
}
