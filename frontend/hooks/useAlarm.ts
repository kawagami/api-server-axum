"use client";

import { useState, useCallback } from 'react';
import useCountdownCore from './useCountdownCore';

export default function useAlarm() {
    const now = new Date();
    const [hour, setHour] = useState(now.getHours());
    const [minute, setMinute] = useState(now.getMinutes());
    const { targetTime, timeLeft, isRunning, isBeeping, startAt, clear, stopBeeping } = useCountdownCore();

    const startAlarm = useCallback(() => {
        const target = new Date();
        target.setHours(hour, minute, 0, 0);
        // 設定時刻已過 → 定到明天同一時刻
        if (target.getTime() <= Date.now()) {
            target.setDate(target.getDate() + 1);
        }
        startAt(target.getTime());
    }, [hour, minute, startAt]);

    const resetAlarm = useCallback(() => clear(0), [clear]);

    return {
        hour,
        setHour,
        minute,
        setMinute,
        timeLeft,
        targetTime,
        isRunning,
        isBeeping,
        startAlarm,
        resetAlarm,
        stopBeeping,
    };
}
