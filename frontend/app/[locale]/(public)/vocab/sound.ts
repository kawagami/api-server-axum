// 單字闖關音效(合成,無音檔)。核心與對戰遊戲共用 @/libs/audio。
import { isAudioMuted, playTones, setAudioMuted, warmupAudio } from "@/libs/audio";

export const vocabSound = {
    setMuted(m: boolean) { setAudioMuted(m); },
    isMuted() { return isAudioMuted(); },
    warmup() { warmupAudio(); }, // 接在「開始」按鈕點擊內,解 autoplay 鎖
    // 答對:清脆上揚兩音;連對越多音越高(每連一題升半音,最多升 12 個半音 = 一個八度)
    correct(combo = 1) {
        const k = Math.pow(2, Math.min(Math.max(combo - 1, 0), 12) / 12);
        playTones([{ freq: 660 * k, dur: 0.08, type: "sine", gain: 0.14 }, { freq: 990 * k, dur: 0.11, type: "sine", gain: 0.14 }]);
    },
    // 連對里程碑(5 / 10 / 20 / 之後每 10):大三和弦琶音
    comboMilestone() {
        playTones([784, 988, 1175, 1568].map((freq, i) => ({ freq, dur: i === 3 ? 0.22 : 0.07, type: "triangle" as const, gain: 0.14 })));
    },
    // 答錯:低沉短促
    wrong() { playTones([{ freq: 196, dur: 0.16, type: "sawtooth", gain: 0.1 }, { freq: 147, dur: 0.14, type: "sawtooth", gain: 0.1 }]); },
    // 升級:上行琶音
    levelUp() { playTones([{ freq: 523, dur: 0.09, type: "sine" }, { freq: 659, dur: 0.09, type: "sine" }, { freq: 784, dur: 0.09, type: "sine" }, { freq: 1047, dur: 0.16, type: "sine" }]); },
    // 生存模式扣命:比一般答錯更重的「碎裂」下滑音
    lifeLost() { playTones([{ freq: 330, dur: 0.07, type: "square", gain: 0.09 }, { freq: 220, dur: 0.09, type: "sawtooth", gain: 0.1 }, { freq: 110, dur: 0.22, type: "sawtooth", gain: 0.1 }]); },
    // 倒數最後 10 秒:每秒一聲輕敲
    tick() { playTones([{ freq: 1000, dur: 0.03, type: "square", gain: 0.05 }]); },
    // 刷新個人紀錄
    newBest() { playTones([523, 784, 1047, 1319, 1568].map((freq, i) => ({ freq, dur: i === 4 ? 0.3 : 0.08, type: "triangle" as const, gain: 0.14 }))); },
    // 時間到:下行三音
    timeUp() { playTones([{ freq: 784, dur: 0.12 }, { freq: 587, dur: 0.12 }, { freq: 392, dur: 0.2 }]); },
};
