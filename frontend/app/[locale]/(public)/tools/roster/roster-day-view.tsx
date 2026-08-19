"use client";

import { useMemo } from "react";
import { useLocale, useTranslations } from "next-intl";
import {
    dayDate,
    isWeekend,
    SHIFT_MORNING,
    SHIFT_NIGHT,
    SHIFT_OFF,
    type RosterEntry,
} from "@/libs/roster";

/**
 * 按日視圖：一張卡一天，直接列出當天早／晚／休的人。
 *
 * 存在的理由是**手機**：31 欄的橫表在手機上要一直橫捲才讀得到一天的人力，
 * 而「今天誰上班」正是最常問的問題。唯讀（要改班回按人視圖）。
 */
export default function RosterDayView({ entries, startDate }: { entries: RosterEntry[]; startDate: string }) {
    const t = useTranslations("Roster");
    const locale = useLocale();
    const days = entries[0]?.shifts.length ?? 0;

    // 起始日期是「牆上日期」，dayDate 用 UTC 午夜建構，格式化時必須跟著指定 UTC
    const dateFormat = useMemo(
        () => new Intl.DateTimeFormat(locale, { month: "numeric", day: "numeric", weekday: "short", timeZone: "UTC" }),
        [locale],
    );

    const groups = [
        { shift: SHIFT_MORNING, label: t("shiftMorning") },
        { shift: SHIFT_NIGHT, label: t("shiftNight") },
        { shift: SHIFT_OFF, label: t("shiftOff") },
    ];

    return (
        <ul className="grid grid-cols-1 sm:grid-cols-2 gap-3 p-4">
            {Array.from({ length: days }, (_, index) => {
                const date = dayDate(startDate, index);
                return (
                    <li
                        key={index}
                        className="rounded-2xl border border-neutral-200 dark:border-neutral-700 bg-white/70 dark:bg-neutral-800/60 p-4"
                    >
                        <p className={`text-sm font-bold mb-2 ${isWeekend(date) ? "text-primary-600 dark:text-primary-400" : ""}`}>
                            {date ? dateFormat.format(date) : t("dayColumn", { n: index + 1 })}
                        </p>
                        <dl className="space-y-1 text-sm">
                            {groups.map(group => {
                                const people = entries.filter(entry => entry.shifts[index] === group.shift);
                                const uncovered = people.length === 0 && group.shift !== SHIFT_OFF;
                                return (
                                    <div key={group.shift} className="flex gap-2">
                                        <dt className="w-12 shrink-0 text-neutral-500">{group.label}</dt>
                                        {/* 0 人是警示語意（橘），跟表尾的人力列同一套 */}
                                        <dd className={uncovered ? "text-orange-600 dark:text-orange-400 font-bold" : ""}>
                                            {people.length ? people.map(p => p.name).join("、") : t("dayNobody")}
                                        </dd>
                                    </div>
                                );
                            })}
                        </dl>
                    </li>
                );
            })}
        </ul>
    );
}
