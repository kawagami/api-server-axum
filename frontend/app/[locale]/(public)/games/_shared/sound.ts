// 對戰遊戲音效。合成核心已抽到 @/libs/audio(與 vocab 共用),此檔只留遊戲專屬音色。
import { isAudioMuted, playTones, setAudioMuted, warmupAudio } from "@/libs/audio";

export const sound = {
    setMuted(m: boolean) { setAudioMuted(m); },
    isMuted() { return isAudioMuted(); },
    warmup() { warmupAudio(); }, // 接在使用者手勢內,解 autoplay 鎖
    move() { playTones([{ freq: 320, dur: 0.06 }]); },
    capture() { playTones([{ freq: 380, dur: 0.05 }, { freq: 200, dur: 0.08 }]); },
    check() { playTones([{ freq: 880, dur: 0.1, type: "sawtooth" }]); },
    gameOver() { playTones([{ freq: 520, dur: 0.12 }, { freq: 392, dur: 0.12 }, { freq: 262, dur: 0.2 }]); },

    // ---- 阿瓦隆 ----
    /// 輪到你行動（組隊／投票／出任務／刺殺）
    yourTurn() { playTones([{ freq: 660, dur: 0.09, type: 'sine', gain: 0.18 }, { freq: 880, dur: 0.16, type: 'sine', gain: 0.18 }]); },
    voteReveal() { playTones([{ freq: 440, dur: 0.05, type: 'triangle' }, { freq: 554, dur: 0.05, type: 'triangle' }, { freq: 659, dur: 0.1, type: 'triangle' }]); },
    cardFlip() { playTones([{ freq: 300, dur: 0.05, type: 'triangle', gain: 0.1 }]); },
    failCard() { playTones([{ freq: 150, dur: 0.14, type: 'sawtooth', gain: 0.08 }]); },
    questSuccess() {
        playTones([523, 659, 784, 1047].map((freq, i) => ({ freq, dur: i === 3 ? 0.25 : 0.09, type: 'triangle' as const })));
    },
    questFail() { playTones([{ freq: 196, dur: 0.16, type: 'sawtooth', gain: 0.08 }, { freq: 147, dur: 0.32, type: 'sawtooth', gain: 0.08 }]); },
    assassinPhase() { playTones([{ freq: 110, dur: 0.35, type: 'sawtooth', gain: 0.1 }, { freq: 104, dur: 0.5, type: 'sawtooth', gain: 0.1 }]); },
    win() {
        playTones([523, 659, 784, 1047].map((freq, i) => ({ freq, dur: i === 3 ? 0.4 : 0.12, type: 'triangle' as const, gain: 0.15 })));
    },
    lose() { playTones([{ freq: 392, dur: 0.18, type: 'triangle' }, { freq: 330, dur: 0.18, type: 'triangle' }, { freq: 262, dur: 0.4, type: 'triangle' }]); },
    chat() { playTones([{ freq: 1200, dur: 0.035, type: 'sine', gain: 0.05 }]); },
};
