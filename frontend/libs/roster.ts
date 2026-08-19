/**
 * 排班工具的純函式與常數（`/tools/roster`）。
 *
 * 上限三個常數**鏡射後端** `backend/src/structs/roster.rs`：在前端先擋掉超量輸入，
 * 使用者才會看到「人數上限 100 位」這種具體訊息。只靠後端 422 的話，前端拿到的是
 * 寫死繁中的訊息（不能直接印給 en / zh-CN 使用者），只能退化成一句通用的「排班失敗」。
 * 改上限要兩邊一起改。
 */

export const MAX_NAMES = 100;
export const MAX_NAME_LEN = 50;
export const MAX_DAYS = 31;

/** 班別字串是 API 契約（後端 `routes/roster.rs` 回傳的原字串），不可改成英文代碼 */
export const SHIFT_MORNING = "早班";
export const SHIFT_NIGHT = "晚班";
export const SHIFT_OFF = "休";

/** 手動微調時的切換順序：早 → 晚 → 休 → 早 */
const SHIFT_CYCLE = [SHIFT_MORNING, SHIFT_NIGHT, SHIFT_OFF];

export function nextShift(shift: string): string {
    const index = SHIFT_CYCLE.indexOf(shift);
    return SHIFT_CYCLE[(index + 1) % SHIFT_CYCLE.length];
}

export const ROSTER_RULES = ["fairness", "morning_heavy", "night_heavy"] as const;
export type RosterRule = (typeof ROSTER_RULES)[number];

/** 後端 `RosterWarning` 的機器碼（後端刻意不回文案，i18n 在前端） */
export const ROSTER_WARNINGS = [
    "understaffed",
    "shift_uncovered",
    "night_to_morning",
    "max_consecutive_exceeded",
] as const;
export type RosterWarning = (typeof ROSTER_WARNINGS)[number];

export interface RosterEntry {
    id: number;
    name: string;
    shifts: string[];
}

export interface RosterPlan {
    morning_slots: number;
    night_slots: number;
    rest_slots: number;
    max_consecutive: number;
}

/**
 * 一次貼上多筆姓名：逗號（半／全形）、分號、頓號、換行、Tab 都當分隔。
 * 空白**不當**分隔（"王 小明" 是一個人）。順序保留、去掉重複與空字串。
 */
export function parseNames(input: string): string[] {
    const seen = new Set<string>();
    return input
        .split(/[\n\r\t,，;；、]+/)
        .map(part => part.trim())
        .filter(name => {
            if (!name || seen.has(name)) return false;
            seen.add(name);
            return true;
        });
}

export interface PersonStat {
    name: string;
    morning: number;
    night: number;
    off: number;
    work: number;
    /** 最長連續上班天數 */
    maxStreak: number;
}

export interface DayStat {
    /** 0-based 天序 */
    index: number;
    morning: number;
    night: number;
    off: number;
    /** 早班或晚班 0 人 —— 舊版排班演算法會排出這種天，統計區的存在就是為了讓它看得見 */
    gap: boolean;
}

export interface RosterStats {
    perPerson: PersonStat[];
    perDay: DayStat[];
    /** 有人力洞的天數 */
    gapDays: number;
    workSpread: number;
}

/** 統計在前端算（不在後端）：手動微調格子之後要即時重算，來回打 API 沒有意義 */
export function rosterStats(entries: RosterEntry[]): RosterStats {
    const days = entries[0]?.shifts.length ?? 0;

    const perPerson = entries.map(entry => {
        let streak = 0;
        let maxStreak = 0;
        const stat: PersonStat = { name: entry.name, morning: 0, night: 0, off: 0, work: 0, maxStreak: 0 };
        for (const shift of entry.shifts) {
            if (shift === SHIFT_OFF) {
                stat.off += 1;
                streak = 0;
                continue;
            }
            if (shift === SHIFT_MORNING) stat.morning += 1;
            else stat.night += 1;
            stat.work += 1;
            streak += 1;
            maxStreak = Math.max(maxStreak, streak);
        }
        stat.maxStreak = maxStreak;
        return stat;
    });

    const perDay: DayStat[] = Array.from({ length: days }, (_, index) => {
        const day: DayStat = { index, morning: 0, night: 0, off: 0, gap: false };
        for (const entry of entries) {
            const shift = entry.shifts[index];
            if (shift === SHIFT_MORNING) day.morning += 1;
            else if (shift === SHIFT_NIGHT) day.night += 1;
            else day.off += 1;
        }
        day.gap = day.morning === 0 || day.night === 0;
        return day;
    });

    const work = perPerson.map(p => p.work);
    return {
        perPerson,
        perDay,
        gapDays: perDay.filter(d => d.gap).length,
        workSpread: work.length ? Math.max(...work) - Math.min(...work) : 0,
    };
}

/**
 * 起始日期 + 天序 → 日期。沒給起始日期回 null（表頭就顯示「第 N 天」）。
 *
 * **一律用 UTC 午夜**：`<input type="date">` 給的是「牆上日期」而不是某個時刻，
 * 用本地午夜建構再拿去格式化，時區一換就會位移一天（同一份班表在不同機器上顯示不同日期）。
 * 呼叫端格式化時要跟著指定 `timeZone: "UTC"`。
 */
export function dayDate(startDate: string, index: number): Date | null {
    if (!startDate) return null;
    const base = new Date(`${startDate}T00:00:00Z`);
    if (Number.isNaN(base.getTime())) return null;
    base.setUTCDate(base.getUTCDate() + index);
    return base;
}

/** 週末（六／日）。有起始日期時表頭會標色 —— 週末人力不同是排班的實務前提 */
export function isWeekend(date: Date | null): boolean {
    return date ? date.getUTCDay() === 0 || date.getUTCDay() === 6 : false;
}

function csvCell(value: string): string {
    return /[",\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value;
}

/**
 * 班表攤成二維字串（第一列表頭、第一欄人員；有起始日期就用 `YYYY-MM-DD` 當欄名）。
 * CSV 與「複製表格」共用同一份資料，只差分隔字元。
 */
export function rosterToRows(
    entries: RosterEntry[],
    startDate: string,
    staffHeader: string,
    dayHeader: (n: number) => string,
): string[][] {
    const days = entries[0]?.shifts.length ?? 0;
    const header = [
        staffHeader,
        ...Array.from({ length: days }, (_, i) => {
            const date = dayDate(startDate, i);
            return date ? date.toISOString().slice(0, 10) : dayHeader(i + 1);
        }),
    ];
    return [header, ...entries.map(entry => [entry.name, ...entry.shifts])];
}

/** 匯出 CSV。開頭補 BOM：沒有 BOM 的話 Excel 會把 UTF-8 中文讀成亂碼 */
export function rosterToCsv(rows: string[][]): string {
    return `\ufeff${rows.map(row => row.map(csvCell).join(",")).join("\r\n")}\r\n`;
}
