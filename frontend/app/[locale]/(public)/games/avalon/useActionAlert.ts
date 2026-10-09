"use client";

import { useEffect, useRef } from 'react';
import { sound } from '../_shared/sound';

/// 輪到自己行動時提醒：響一聲 + 手機震動；分頁在背景時讓標題交替閃爍，切回前景或行動完成就還原。
/// `alert` 是要顯示的提醒文字（null = 不需要行動）；同一段提醒只響一次，`alertKey` 換了才再響。
export function useActionAlert(alertKey: string | null, alert: string | null) {
    const lastKey = useRef<string | null>(null);

    useEffect(() => {
        if (!alertKey || alertKey === lastKey.current) {
            if (!alertKey) lastKey.current = null;
            return;
        }
        lastKey.current = alertKey;
        sound.yourTurn();
        if (!sound.isMuted()) navigator.vibrate?.(150);
    }, [alertKey]);

    useEffect(() => {
        if (!alert) return;
        const original = document.title;
        let timer: ReturnType<typeof setInterval> | null = null;
        let on = false;

        const stop = () => {
            if (timer) clearInterval(timer);
            timer = null;
            document.title = original;
        };
        const sync = () => {
            if (!document.hidden) { stop(); return; }
            if (timer) return;
            timer = setInterval(() => {
                on = !on;
                document.title = on ? `⚔ ${alert}` : original;
            }, 1000);
        };

        sync();
        document.addEventListener('visibilitychange', sync);
        return () => {
            document.removeEventListener('visibilitychange', sync);
            stop();
        };
    }, [alert]);
}
