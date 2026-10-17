"use client";

import { useState } from "react";
import { useLocale, useTranslations } from "next-intl";
import { cn } from "@/libs/cn";
import { formatAmount } from "../model";
import Pills from "./pills";

export interface RankingRow {
    key: string;
    name: string;
    /** 名稱下的小字（品項排行的店家） */
    sub?: string | null;
    count: number;
    total: number;
}

type SortKey = "count" | "total";

interface Props {
    className: string;
    title: string;
    rows: RankingRow[];
    /** 次數的單位文案（「去 3 次」/「吃 3 次」） */
    countText: (n: number) => string;
}

const TOP = 10;

/** 前 N 名，可切依次數 / 依金額；長條長度跟著排序依據走 */
export default function Ranking({ className, title, rows, countText }: Props) {
    const t = useTranslations("FoodLogStats");
    const locale = useLocale();
    const [sort, setSort] = useState<SortKey>("count");
    const [expanded, setExpanded] = useState(false);

    // 後端已依次數排好；依金額時同額再比次數
    const sorted = sort === "count" ? rows : [...rows].sort((a, b) => b.total - a.total || b.count - a.count);
    const shown = expanded ? sorted : sorted.slice(0, TOP);
    const max = Math.max(1, ...sorted.map(r => r[sort]));

    return (
        <section className={cn(className, "p-4 flex flex-col gap-3")}>
            <header className="flex items-center justify-between gap-2 flex-wrap">
                <h3 className="font-semibold">{title}</h3>
                <Pills
                    options={[
                        { value: "count", label: t("sortCount") },
                        { value: "total", label: t("sortTotal") },
                    ]}
                    value={sort}
                    onChange={setSort}
                />
            </header>

            {rows.length === 0 ? (
                <p className="text-sm text-center text-neutral-500 dark:text-neutral-400 py-6">{t("empty")}</p>
            ) : (
                <ol className="flex flex-col gap-2.5">
                    {shown.map((r, i) => {
                        const primary = sort === "count" ? countText(r.count) : formatAmount(r.total, locale);
                        const secondary = sort === "count" ? formatAmount(r.total, locale) : countText(r.count);
                        return (
                            <li key={r.key} className="flex items-start gap-3">
                                <span className="w-5 shrink-0 text-right text-sm tabular-nums text-neutral-400 dark:text-neutral-500">
                                    {i + 1}
                                </span>
                                <div className="flex-1 min-w-0 flex flex-col gap-1">
                                    <div className="flex items-baseline justify-between gap-2">
                                        <span className="truncate">
                                            {r.name}
                                            {r.sub && (
                                                <span className="ml-1.5 text-xs text-neutral-500 dark:text-neutral-400">{r.sub}</span>
                                            )}
                                        </span>
                                        <span className="shrink-0 text-sm tabular-nums">
                                            <span className="font-semibold">{primary}</span>
                                            <span className="ml-2 text-neutral-500 dark:text-neutral-400">{secondary}</span>
                                        </span>
                                    </div>
                                    <div className="h-1.5 rounded-full bg-neutral-100 dark:bg-neutral-700/60">
                                        <div
                                            className="h-full rounded-full bg-primary-500"
                                            style={{ width: `${(r[sort] / max) * 100}%` }}
                                        />
                                    </div>
                                </div>
                            </li>
                        );
                    })}
                </ol>
            )}

            {rows.length > TOP && (
                <button
                    type="button"
                    onClick={() => setExpanded(e => !e)}
                    className="self-center px-3 py-1 text-sm rounded-sm text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 transition-colors"
                >
                    {expanded ? t("showTop", { n: TOP }) : t("showAll", { n: rows.length })}
                </button>
            )}
        </section>
    );
}
