"use client";

import { useEffect, useRef } from 'react';

export function useAudioBeeper(
    isBeeping: boolean,
    stopBeeping: () => void,
    onVisibilityReset: () => void,
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
        const handleVisibility = () => {
            if (document.visibilityState === 'visible' && isBeeping) {
                onVisibilityReset();
            }
        };
        document.addEventListener('visibilitychange', handleVisibility);
        return () => document.removeEventListener('visibilitychange', handleVisibility);
    }, [isBeeping, onVisibilityReset]);

    return audioRef;
}
