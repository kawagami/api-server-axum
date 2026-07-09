// 單字闖關音效(合成,無音檔)。核心與對戰遊戲共用 @/libs/audio。
import { isAudioMuted, playTones, setAudioMuted, warmupAudio } from "@/libs/audio";

export const vocabSound = {
    setMuted(m: boolean) { setAudioMuted(m); },
    isMuted() { return isAudioMuted(); },
    warmup() { warmupAudio(); }, // 接在「開始」按鈕點擊內,解 autoplay 鎖
    // 答對:清脆上揚兩音
    correct() { playTones([{ freq: 660, dur: 0.08, type: "sine", gain: 0.14 }, { freq: 990, dur: 0.11, type: "sine", gain: 0.14 }]); },
    // 答錯:低沉短促
    wrong() { playTones([{ freq: 196, dur: 0.16, type: "sawtooth", gain: 0.1 }, { freq: 147, dur: 0.14, type: "sawtooth", gain: 0.1 }]); },
    // 升級:上行琶音
    levelUp() { playTones([{ freq: 523, dur: 0.09, type: "sine" }, { freq: 659, dur: 0.09, type: "sine" }, { freq: 784, dur: 0.09, type: "sine" }, { freq: 1047, dur: 0.16, type: "sine" }]); },
    // 時間到:下行三音
    timeUp() { playTones([{ freq: 784, dur: 0.12 }, { freq: 587, dur: 0.12 }, { freq: 392, dur: 0.2 }]); },
};
