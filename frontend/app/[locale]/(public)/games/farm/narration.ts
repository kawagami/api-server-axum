// 語音導覽的台詞組裝（純函式）：最新一批事件（別人做了什麼 / 收穫 / 換輪）＋「現在輪到誰、要做什麼」。
// 只講狀態不給策略建議。畫面上的字幕與「再聽一次」念的都是這段。

import { HARVEST_ROUNDS, type FarmState } from './farm-types';
import type { FarmEvent } from './useFarmRoom';
import { feedingOutlook } from './farm-rules';
import { ACCUM_RES } from './farm-ui';

type T = (key: string, values?: Record<string, string | number>) => string;

const resName = (t: T, action: string) => {
    const r = ACCUM_RES[action];
    return r === 'sheep' || r === 'boar' || r === 'cattle' ? t(`animal_${r}`) : t(`res_${r}`);
};

/** 輪到誰、要做什麼（輪到自己且本輪末收穫時，順帶報糧食狀況） */
export function turnSentence(t: T, state: FarmState, mySeat: number | null, seatName: (s: number) => string): string {
    if (state.phase === 'game_over') return '';
    const cur = state.current_player;
    if (cur === null) return t('settling');
    if (cur !== mySeat) return t('voice_otherTurn', { name: seatName(cur) });

    const parts = [t('voice_yourTurn', { n: state.workers_left[cur] ?? 0 })];
    const me = state.players[cur];
    if (me && HARVEST_ROUNDS.has(state.round)) {
        const f = feedingOutlook(me);
        parts.push(f.shortfall > 0 ? t('voice_harvestShort', { need: f.need, food: f.food, n: f.shortfall })
            : f.food < f.need ? t('voice_harvestConvert', { need: f.need, food: f.food })
                : t('voice_harvestOk', { need: f.need, food: f.food }));
    }
    return parts.join(' ');
}

export function composeNarration(
    t: T, state: FarmState, events: FarmEvent[], mySeat: number | null, seatName: (s: number) => string,
): string {
    const parts: string[] = [];
    for (const e of events) {
        if (e.kind === 'action') {
            if (e.seat === mySeat) continue; // 自己剛做的不用念
            const vars = { name: seatName(e.seat), action: t(`action_${e.action}`) };
            parts.push(e.amount > 0 ? t('voice_tookAmount', { ...vars, amount: e.amount, res: resName(t, e.action) }) : t('voice_took', vars));
        } else if (e.kind === 'harvest') {
            parts.push(t('voice_harvestDone', { n: e.round }));
            const mine = mySeat !== null ? e.begging[mySeat] ?? 0 : 0;
            if (mine > 0) parts.push(t('voice_youBegged', { n: mine }));
        } else {
            parts.push(t('voice_roundStart', { n: e.round }));
        }
    }
    const turn = turnSentence(t, state, mySeat, seatName);
    if (turn) parts.push(turn);
    return parts.join(' ');
}
