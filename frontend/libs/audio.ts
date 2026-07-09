// Web Audio 合成音效核心（無音檔),games 與 vocab 共用。
// AudioContext 必須在使用者手勢後建立/resume(瀏覽器 autoplay 政策),故對外提供 warmup()。
// 靜音偏好持久化於 localStorage,games / vocab 共享單一設定。

export type Tone = { freq: number; dur: number; type?: OscillatorType; gain?: number };

const MUTE_KEY = "sound_muted";
let ctx: AudioContext | null = null;
let muted = false;
let muteLoaded = false;

function loadMute(): void {
    if (muteLoaded || typeof window === "undefined") return;
    muteLoaded = true;
    muted = window.localStorage.getItem(MUTE_KEY) === "1";
}

function ac(): AudioContext | null {
    if (typeof window === "undefined") return null;
    if (!ctx) {
        const AC = window.AudioContext
            ?? (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
        if (!AC) return null;
        ctx = new AC();
    }
    if (ctx.state === "suspended") void ctx.resume();
    return ctx;
}

export function playTones(tones: Tone[]): void {
    loadMute();
    if (muted) return;
    const c = ac();
    if (!c) return;
    let t = c.currentTime;
    for (const tone of tones) {
        const osc = c.createOscillator();
        const g = c.createGain();
        osc.type = tone.type ?? "square";
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

/** 接在使用者手勢內,解 autoplay 鎖 */
export function warmupAudio(): void {
    ac();
}

export function setAudioMuted(m: boolean): void {
    muted = m;
    muteLoaded = true;
    if (typeof window !== "undefined") {
        window.localStorage.setItem(MUTE_KEY, m ? "1" : "0");
    }
}

export function isAudioMuted(): boolean {
    loadMute();
    return muted;
}
