// Web Audio 合成音效（無音檔）。三遊戲共用。
// AudioContext 必須在使用者手勢後建立/resume（瀏覽器 autoplay 政策）。

type Tone = { freq: number; dur: number; type?: OscillatorType; gain?: number };

let ctx: AudioContext | null = null;
let muted = false;

function ac(): AudioContext | null {
    if (typeof window === 'undefined') return null;
    if (!ctx) {
        const AC = window.AudioContext ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
        if (!AC) return null;
        ctx = new AC();
    }
    if (ctx.state === 'suspended') void ctx.resume();
    return ctx;
}

function play(tones: Tone[]): void {
    if (muted) return;
    const c = ac();
    if (!c) return;
    let t = c.currentTime;
    for (const tone of tones) {
        const osc = c.createOscillator();
        const g = c.createGain();
        osc.type = tone.type ?? 'square';
        osc.frequency.value = tone.freq;
        const peak = tone.gain ?? 0.12;
        g.gain.setValueAtTime(0.0001, t);
        g.gain.exponentialRampToValueAtTime(peak, t + 0.005);
        g.gain.exponentialRampToValueAtTime(0.0001, t + tone.dur);
        osc.connect(g);
        g.connect(c.destination);
        osc.start(t);
        osc.stop(t + tone.dur);
        t += tone.dur;
    }
}

export const sound = {
    setMuted(m: boolean) { muted = m; },
    isMuted() { return muted; },
    warmup() { ac(); }, // 接在使用者手勢內，解 autoplay 鎖
    move() { play([{ freq: 320, dur: 0.06 }]); },
    capture() { play([{ freq: 380, dur: 0.05 }, { freq: 200, dur: 0.08 }]); },
    check() { play([{ freq: 880, dur: 0.1, type: 'sawtooth' }]); },
    gameOver() { play([{ freq: 520, dur: 0.12 }, { freq: 392, dur: 0.12 }, { freq: 262, dur: 0.2 }]); },
};
