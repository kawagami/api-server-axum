// 農場畫面的純展示常數：圖示、行動格分組、座位色。不含規則（規則鏡射在 farm-rules.ts）。
// ⚠️ emoji 只用 Emoji 11 以前的：Windows 10 的 Segoe UI Emoji 不認得 13 代（🪵🪨🛖），會顯示成方框。

export const RES_KEYS = ['wood', 'clay', 'reed', 'stone', 'grain', 'veg', 'food'] as const;
export const ANIMAL_KEYS = ['sheep', 'boar', 'cattle'] as const;

export const RES_ICON: Record<(typeof RES_KEYS)[number] | (typeof ANIMAL_KEYS)[number], string> = {
    wood: '🌳', clay: '🧱', reed: '🎋', stone: '⛰️', grain: '🌾', veg: '🥕', food: '🍲',
    sheep: '🐑', boar: '🐗', cattle: '🐄',
};

export const ACTION_ICON: Record<string, string> = {
    forest: '🌲', clay_pit: '🧱', reed: '🎋', quarry: '⛏️', river: '🎣',
    sheep_pen: '🐑', boar_pen: '🐗', cattle_pen: '🐄',
    grain_seeds: '🌾', veg_seeds: '🥕', plow: '🚜', sow: '🌱', fences: '🌿',
    build_rooms: '🏠', renovate: '🔨', family_growth: '👶',
    day_labor: '💪', start_player: '🥇',
};

/** 累積格取走的是哪種資源（徽章圖示用） */
export const ACCUM_RES: Record<string, keyof typeof RES_ICON> = {
    forest: 'wood', clay_pit: 'clay', reed: 'reed', quarry: 'stone', river: 'food',
    sheep_pen: 'sheep', boar_pen: 'boar', cattle_pen: 'cattle',
};

/** 行動板分組（涵蓋 wire 的 18 種 action） */
export const ACTION_GROUPS: { key: string; actions: string[] }[] = [
    { key: 'materials', actions: ['forest', 'clay_pit', 'reed', 'quarry', 'river'] },
    { key: 'animals', actions: ['sheep_pen', 'boar_pen', 'cattle_pen'] },
    { key: 'farmland', actions: ['plow', 'grain_seeds', 'veg_seeds', 'sow', 'fences'] },
    { key: 'home', actions: ['build_rooms', 'renovate', 'family_growth'] },
    { key: 'other', actions: ['day_labor', 'start_player'] },
];

export const HOUSE_ICON: Record<string, string> = { wood: '🏡', clay: '🏠', stone: '🏰' };

/** 座位色（遊戲資產色，同棋子色寫死；seat 0–3） */
export const SEAT_COLORS = [
    { dot: 'bg-sky-500', border: 'border-sky-500', text: 'text-sky-600 dark:text-sky-400' },
    { dot: 'bg-rose-500', border: 'border-rose-500', text: 'text-rose-600 dark:text-rose-400' },
    { dot: 'bg-amber-500', border: 'border-amber-500', text: 'text-amber-600 dark:text-amber-400' },
    { dot: 'bg-violet-500', border: 'border-violet-500', text: 'text-violet-600 dark:text-violet-400' },
] as const;

export const seatColor = (seat: number) => SEAT_COLORS[seat % SEAT_COLORS.length];
