/** 排行區間可選的天數（後端上限 366） */
export const RANGE_OPTIONS = [30, 90, 365] as const;
export const DEFAULT_RANGE = RANGE_OPTIONS[0];

/** 平均（四捨五入到整數元）；分母是「有紀錄的天數」，0 天回 0 */
export function average(total: number, days: number): number {
    return days > 0 ? Math.round(total / days) : 0;
}

/** 本月至今 vs 上月同期：差額與百分比（上月同期 0 元時百分比沒有意義，回 null） */
export function monthChange(current: number, previous: number): { diff: number; percent: number | null } {
    const diff = current - previous;
    return { diff, percent: previous > 0 ? Math.round((diff / previous) * 100) : null };
}
