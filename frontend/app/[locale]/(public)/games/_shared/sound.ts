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
};
