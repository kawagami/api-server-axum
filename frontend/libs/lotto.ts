import type { LottoGame, LottoNote, LottoPrizeTier } from "@/types";

export interface GameSpec {
    mainMax: number; // 一般號/第一區上限
    mainCount: number; // 須選個數
    hasSecond: boolean; // 是否有第二區
    secondMax: number; // 第二區上限（無第二區時 0）
}

// 大樂透 1–49 選 6；威力彩第一區 1–38 選 6 + 第二區 1–8 選 1
export const GAME_SPECS: Record<LottoGame, GameSpec> = {
    lotto649: { mainMax: 49, mainCount: 6, hasSecond: false, secondMax: 0 },
    super_lotto638: { mainMax: 38, mainCount: 6, hasSecond: true, secondMax: 8 },
};

export const LOTTO_GAMES: LottoGame[] = ['lotto649', 'super_lotto638'];

// i18n key 對照（彩種 / 獎別）
export const GAME_KEY: Record<LottoGame, string> = {
    lotto649: 'gameLotto649',
    super_lotto638: 'gameSuperLotto638',
};

export const PRIZE_KEY: Record<LottoPrizeTier, string> = {
    first: 'prizeFirst',
    second: 'prizeSecond',
    third: 'prizeThird',
    fourth: 'prizeFourth',
    fifth: 'prizeFifth',
    sixth: 'prizeSixth',
    seventh: 'prizeSeventh',
    eighth: 'prizeEighth',
    ninth: 'prizeNinth',
    general: 'prizeGeneral',
};

// 開獎星期：大樂透週二(2)/五(5)，威力彩週一(1)/四(4)
const DRAW_WEEKDAYS: Record<LottoGame, number[]> = {
    lotto649: [2, 5],
    super_lotto638: [1, 4],
};

// 下一期開獎日（含今天，若今天就是開獎日），以台北時區計算，回 YYYY-MM-DD
export function nextDrawDate(game: LottoGame, now: Date = new Date()): string {
    const todayStr = new Intl.DateTimeFormat('en-CA', {
        timeZone: 'Asia/Taipei',
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
    }).format(now); // YYYY-MM-DD（台北）
    // 以 UTC 午夜做星期運算，避免瀏覽器時區影響
    const base = new Date(`${todayStr}T00:00:00Z`);
    const days = DRAW_WEEKDAYS[game];
    for (let i = 0; i <= 7; i++) {
        const d = new Date(base);
        d.setUTCDate(d.getUTCDate() + i);
        if (days.includes(d.getUTCDay())) return d.toISOString().slice(0, 10);
    }
    return todayStr;
}

// 一注是否完整合法（前端先擋，後端會再驗）
export function isNoteValid(game: LottoGame, note: LottoNote): boolean {
    const spec = GAME_SPECS[game];
    const { picks, second } = note;
    if (picks.length !== spec.mainCount) return false;
    if (new Set(picks).size !== picks.length) return false;
    if (picks.some(n => !Number.isInteger(n) || n < 1 || n > spec.mainMax)) return false;
    if (spec.hasSecond) {
        if (second === null || !Number.isInteger(second) || second < 1 || second > spec.secondMax) return false;
    } else if (second !== null) {
        return false;
    }
    return true;
}

// 隨機快選一注（電腦選號）
export function quickPick(game: LottoGame): LottoNote {
    const spec = GAME_SPECS[game];
    const pool = Array.from({ length: spec.mainMax }, (_, i) => i + 1);
    // Fisher–Yates 洗牌後取前 mainCount
    for (let i = pool.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [pool[i], pool[j]] = [pool[j], pool[i]];
    }
    const picks = pool.slice(0, spec.mainCount).sort((a, b) => a - b);
    const second = spec.hasSecond ? Math.floor(Math.random() * spec.secondMax) + 1 : null;
    return { picks, second };
}
