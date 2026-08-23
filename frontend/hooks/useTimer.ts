"use client";

import { useState, useCallback } from 'react';
import useCountdownCore from './useCountdownCore';

export default function useTimer() {
    const [minutes, setMinutes] = useState(30);
    const { targetTime, timeLeft, isRunning, isBeeping, startAt, pause, clear, stopBeeping } = useCountdownCore(minutes * 60);
    const isPaused = !isRunning && targetTime !== null;

    const updateMinutes = useCallback((m: number) => {
        setMinutes(m);
        // 閒置時改分鐘數要同步顯示；跑到一半（含暫停）改設定不動當前剩餘秒數
        if (!isRunning && targetTime === null) {
            clear(m * 60);
        }
    }, [isRunning, targetTime, clear]);

    const startCountdown = useCallback(() => {
        const now = Date.now();
        if (targetTime === null) {
            startAt(now + minutes * 60 * 1000);
        } else if (isPaused) {
            startAt(now + timeLeft * 1000);
        }
    }, [targetTime, minutes, timeLeft, isPaused, startAt]);

    const pauseCountdown = useCallback(() => pause(), [pause]);

    const resetCountdown = useCallback(() => clear(minutes * 60), [clear, minutes]);

    return {
        minutes,
        setMinutes: updateMinutes,
        timeLeft,
        targetTime,
        isRunning,
        isPaused,
        isBeeping,
        startCountdown,
        pauseCountdown,
        resetCountdown,
        stopBeeping,
    };
}
