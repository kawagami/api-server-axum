"use client";

import { useMemo } from "react";
import { useLocale, useTranslations } from "next-intl";
import ShiftBadge from "./shift-badge";
import {
    dayDate,
    isWeekend,
    rosterStats,
    SHIFT_MORNING,
    SHIFT_NIGHT,
    type RosterEntry,
} from "@/libs/roster";

interface Props {
    entries: RosterEntry[];
    /** 空字串 = 沒指定起始日期，表頭顯示「第 N 天」 */
    startDate: string;
    /** 給了才可編輯：點格子切換班別 */
    onToggle?: (personIndex: number, dayIndex: number) => void;
}

/**
 * 按人視圖：一列一人、一欄一天。**這裡是唯一可編輯的視圖**（按日視圖唯讀）——
 * 實務班表一定要手動換班，只能重新生成的話等於整份重來。
 *
 * 表尾常駐每日早／晚班人數：人力洞（某班 0 人）在格子海裡看不出來，
 * 一列數字才看得見，這也是舊版演算法最嚴重的缺陷（morning_heavy 3 人的第 4 天晚班沒人）。
 */
export default function RosterTable({ entries, startDate, onToggle }: Props) {
    const t = useTranslations("Roster");
    const locale = useLocale();
    const days = entries[0]?.shifts.length ?? 0;
    const stats = useMemo(() => rosterStats(entries), [entries]);

    // 起始日期是「牆上日期」，dayDate 用 UTC 午夜建構，格式化時必須跟著指定 UTC
    const dateFormat = useMemo(
        () => new Intl.DateTimeFormat(locale, { month: "numeric", day: "numeric", timeZone: "UTC" }),
        [locale],
    );
    const weekdayFormat = useMemo(
        () => new Intl.DateTimeFormat(locale, { weekday: "short", timeZone: "UTC" }),
        [locale],
    );

    const columns = useMemo(
        () =>
            Array.from({ length: days }, (_, i) => {
                const date = dayDate(startDate, i);
                return {
                    key: i,
                    label: date ? dateFormat.format(date) : t("dayColumn", { n: i + 1 }),
                    weekday: date ? weekdayFormat.format(date) : null,
                    weekend: isWeekend(date),
                };
            }),
        [days, startDate, dateFormat, weekdayFormat, t],
    );

    const stickyCell = "sticky left-0 bg-white/95 dark:bg-neutral-800/95";

    return (
        <div className="overflow-x-auto">
            <table className="w-full text-left border-collapse">
                <caption className="sr-only">{t("resultHeading")}</caption>
                <thead>
                    <tr className="bg-neutral-100/50 dark:bg-neutral-800/50">
                        <th scope="col" className={`p-4 border-b dark:border-neutral-700 font-semibold z-20 ${stickyCell}`}>
                            {t("staffColumn")}
                        </th>
                        {columns.map(col => (
                            <th
                                key={col.key}
                                scope="col"
                                className={`p-2 border-b dark:border-neutral-700 font-semibold text-center min-w-[76px] ${col.weekend ? "text-primary-600 dark:text-primary-400" : ""}`}
                            >
                                <div className="text-xs font-mono">{col.label}</div>
                                {col.weekday && <div className="text-[10px] text-neutral-500">{col.weekday}</div>}
                            </th>
                        ))}
                    </tr>
                </thead>
                <tbody>
                    {entries.map((staff, personIndex) => (
                        <tr key={staff.id} className="hover:bg-white/40 dark:hover:bg-neutral-800/40 transition-colors">
                            <th scope="row" className={`p-4 border-b dark:border-neutral-700 font-bold text-left z-10 ${stickyCell}`}>
                                {staff.name}
                            </th>
                            {staff.shifts.map((shift, dayIndex) => (
                                <td key={dayIndex} className="p-2 border-b dark:border-neutral-700 text-center">
                                    {onToggle ? (
                                        <button
                                            type="button"
                                            onClick={() => onToggle(personIndex, dayIndex)}
                                            aria-label={`${staff.name} ${columns[dayIndex]?.label} ${shift}`}
                                            className="rounded-full hover:opacity-80 transition-opacity"
                                        >
                                            <ShiftBadge type={shift} />
                                        </button>
                                    ) : (
                                        <ShiftBadge type={shift} />
                                    )}
                                </td>
                            ))}
                        </tr>
                    ))}
                </tbody>
                <tfoot className="text-xs">
                    {[
                        { label: t("shiftMorning"), pick: (d: (typeof stats.perDay)[number]) => d.morning, shift: SHIFT_MORNING },
                        { label: t("shiftNight"), pick: (d: (typeof stats.perDay)[number]) => d.night, shift: SHIFT_NIGHT },
                    ].map(row => (
                        <tr key={row.shift} className="bg-neutral-100/50 dark:bg-neutral-800/50">
                            <th scope="row" className={`p-3 font-semibold text-left text-neutral-600 dark:text-neutral-300 z-10 ${stickyCell}`}>
                                {row.label}
                            </th>
                            {stats.perDay.map(day => (
                                <td
                                    key={day.index}
                                    // 0 人是警示語意（橘），跟班別 badge 的未知值同一套
                                    className={`p-3 text-center font-mono ${row.pick(day) === 0 ? "text-orange-600 dark:text-orange-400 font-bold" : "text-neutral-500"}`}
                                >
                                    {row.pick(day)}
                                </td>
                            ))}
                        </tr>
                    ))}
                </tfoot>
            </table>
        </div>
    );
}
