"use client";

import { useCallback, useEffect, useState } from 'react';
import { canSpeak, speakText, stopSpeaking } from '@/libs/speech';

const VOICE_OFF_KEY = 'farm_voice_off';

function loadVoiceOff(): boolean {
    try { return window.localStorage.getItem(VOICE_OFF_KEY) === '1'; } catch { return false; }
}

/// 語音導覽：自動念（可關，偏好存 localStorage）＋ 手動「再聽一次」（關掉自動念也能用）。
/// FarmPlay 只在收到 WS state 後才 render（不經 SSR），所以初值可以直接讀 window。
export function useFarmVoice(lang: string) {
    const [supported] = useState(canSpeak);
    const [enabled, setEnabled] = useState(() => !loadVoiceOff());

    const auto = useCallback((text: string) => {
        if (enabled) speakText(text, lang);
    }, [enabled, lang]);

    const replay = useCallback((text: string) => speakText(text, lang), [lang]);

    const toggle = useCallback(() => {
        const next = !enabled;
        setEnabled(next);
        try { window.localStorage.setItem(VOICE_OFF_KEY, next ? '0' : '1'); } catch { /* 私密模式等，只是不記住 */ }
        if (!next) stopSpeaking();
    }, [enabled]);

    // 離開對局畫面就閉嘴（共用 socket 不斷線，語音不會自己停）
    useEffect(() => stopSpeaking, []);

    return { supported, enabled, toggle, auto, replay };
}
