// 前端鏡射的規則數字：只用來「預覽與提早擋按鈕」，最終仍由 server 判定（擋錯了 server 回 error 也會顯示）。
// 花費 / 收穫需求出自 protocol/games-wire.md「四、農場經營」；煮食換算與 harvest 順序出自
// backend/src/games/farm/engine.rs 的 harvest()。改後端規則要一併改這裡。

import { HARVEST_ROUNDS, TOTAL_ROUNDS, type PlayerState } from './farm-types';

/** 下一個收穫輪（含本輪）；null = 已無收穫 */
export function nextHarvest(round: number): number | null {
    for (let r = round; r <= TOTAL_ROUNDS; r++) if (HARVEST_ROUNDS.has(r)) return r;
    return null;
}

/** 收穫時的餵食估算：需求、手上糧、可自動煮掉補上的量（穀/菜 1、羊豬 2、牛 3，含本次田收成） */
export function feedingOutlook(p: PlayerState) {
    const need = p.family * 2;
    const harvestYield = p.fields.filter(f => f !== null).length;
    const convertible = p.grain + p.veg + harvestYield + (p.sheep + p.boar) * 2 + p.cattle * 3;
    const shortfall = Math.max(0, need - p.food - convertible);
    return { need, food: p.food, convertible, shortfall };
}

const houseMaterial = (p: PlayerState) => (p.house === 'wood' ? p.wood : p.house === 'clay' ? p.clay : p.stone);

/** 複合動作的花費（單位：資源 key → 數量） */
export function compoundCost(action: string, p: PlayerState, form: Record<string, number | boolean>): Partial<Record<'wood' | 'clay' | 'stone' | 'reed' | 'grain' | 'veg', number>> {
    const n = (k: string) => Number(form[k] ?? 0);
    if (action === 'sow') return { grain: n('grain_fields'), veg: n('veg_fields') };
    if (action === 'build_rooms') {
        const mat = 5 * n('rooms');
        const cost: Partial<Record<'wood' | 'clay' | 'stone' | 'reed', number>> = { reed: 2 * n('rooms') };
        cost[p.house] = mat;
        cost.wood = (cost.wood ?? 0) + 2 * n('stables');
        return cost;
    }
    if (action === 'fences') return { wood: n('pasture_tiles') + 1 + (form.pasture_stable ? 2 : 0) };
    return {};
}

/** 複合表單的錯誤（對應 err_* key）；null = 可送出。全 0 也擋（白放一個工人） */
export function compoundError(action: string, p: PlayerState, form: Record<string, number | boolean>): string | null {
    const n = (k: string) => Number(form[k] ?? 0);
    const emptyFields = p.fields.filter(f => f === null).length;
    if (action === 'sow') {
        if (n('grain_fields') + n('veg_fields') === 0) return 'err_nothing_chosen';
        if (n('grain_fields') + n('veg_fields') > emptyFields) return 'err_not_enough_fields';
        if (n('grain_fields') > p.grain || n('veg_fields') > p.veg) return 'err_no_seeds';
    } else if (action === 'build_rooms') {
        if (n('rooms') + n('stables') === 0) return 'err_nothing_chosen';
        if (n('rooms') + n('stables') > p.free_tiles) return 'err_no_space';
        const cost = compoundCost(action, p, form);
        if (houseMaterial(p) < (cost[p.house] ?? 0) || p.reed < (cost.reed ?? 0) || p.wood < (cost.wood ?? 0)) return 'err_no_materials';
    } else if (action === 'fences') {
        if (n('pasture_tiles') < 1) return 'err_bad_pasture';
        if (n('pasture_tiles') > p.free_tiles) return 'err_no_space';
        if (p.wood < (compoundCost(action, p, form).wood ?? 0)) return 'err_no_wood';
    }
    return null;
}

/** 這格現在一定做不了的原因（err_* key）；null = 可以放。只擋確定失敗的，不做策略判斷 */
export function blockedReason(action: string, p: PlayerState): string | null {
    const emptyFields = p.fields.filter(f => f === null).length;
    switch (action) {
        case 'plow': return p.free_tiles < 1 ? 'err_no_space' : null;
        case 'sow':
            if (emptyFields === 0) return 'err_not_enough_fields';
            return p.grain + p.veg === 0 ? 'err_no_seeds' : null;
        case 'fences':
            if (p.free_tiles < 1) return 'err_no_space';
            return p.wood < 2 ? 'err_no_wood' : null;
        case 'build_rooms': {
            if (p.free_tiles < 1) return 'err_no_space';
            const canRoom = houseMaterial(p) >= 5 && p.reed >= 2;
            return canRoom || p.wood >= 2 ? null : 'err_no_materials';
        }
        case 'renovate':
            if (p.house === 'stone') return 'err_max_house';
            return (p.house === 'wood' ? p.clay : p.stone) >= p.rooms && p.reed >= 1 ? null : 'err_no_materials';
        case 'family_growth': return p.family >= p.rooms ? 'err_no_room' : null;
        default: return null;
    }
}

/** 複合表單各欄位的上限（步進器用） */
export function compoundMax(action: string, field: string, p: PlayerState): number {
    const emptyFields = p.fields.filter(f => f === null).length;
    if (action === 'sow') return Math.min(emptyFields, field === 'grain_fields' ? p.grain : p.veg);
    if (action === 'build_rooms') {
        if (field === 'rooms') return Math.min(p.free_tiles, Math.floor(houseMaterial(p) / 5), Math.floor(p.reed / 2));
        return Math.min(p.free_tiles, Math.floor(p.wood / 2));
    }
    if (action === 'fences') return Math.max(1, Math.min(p.free_tiles, p.wood - 1));
    return 0;
}
