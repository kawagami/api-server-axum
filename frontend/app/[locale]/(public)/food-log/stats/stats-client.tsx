"use client";

import { useState, useTransition } from "react";
import { useLocale, useTranslations } from "next-intl";
import { Minus, TrendingDown, TrendingUp } from "lucide-react";
import Toast, { useToast } from "@/components/toast";
import { getFoodLogStats } from "@/api/food-log";
import type { FoodLogStats } from "@/types";
import { formatAmount } from "../model";
import SpendChart, { type SpendBar } from "./spend-chart";
import Ranking from "./ranking";
import Pills from "./pills";
import { RANGE_OPTIONS, average, monthChange } from "./stats-model";

interface Props {
    initial: FoodLogStats;
    initialDays: number;
}

const CARD = "bg-white dark:bg-neutral-800 rounded-xl shadow-sm border dark:border-neutral-700";

export default function StatsClient({ initial, initialDays }: Props) {
    const t = useTranslations("FoodLogStats");
    const locale = useLocale();
    const { toast, showToast } = useToast();
    const [isPending, startTransition] = useTransition();
    const [stats, setStats] = useState(initial);
    const [days, setDays] = useState(initialDays);

    const money = (n: number) => formatAmount(n, locale);

    function changeRange(next: number) {
        startTransition(async () => {
            const result = await getFoodLogStats(next);
            if (!result.ok) {
                showToast("error", t("loadFailed"));
                return;
            }
            setStats(result.data);
            setDays(next);
        });
    }

    // 日期字串沒有時區：取台北正午格式化，避免任何時區換算跨日（同 day-card.tsx）
    const dayFmt = new Intl.DateTimeFormat(locale, { month: "numeric", day: "numeric", weekday: "short", timeZone: "Asia/Taipei" });
    const monthFmt = new Intl.DateTimeFormat(locale, { month: "short", timeZone: "Asia/Taipei" });
    const monthLongFmt = new Intl.DateTimeFormat(locale, { year: "numeric", month: "long", timeZone: "Asia/Taipei" });
    const noon = (date: string) => new Date(`${date}T12:00:00+08:00`);

    const { daily, monthly, month_compare: compare, overview } = stats;

    const recordedDaily = daily.filter(d => d.recorded);
    const dailyAvg = average(recordedDaily.reduce((s, d) => s + d.total, 0), recordedDaily.length);
    const dailyBars: SpendBar[] = daily.map((d, i) => {
        const date = dayFmt.format(noon(d.date));
        return {
            key: d.date,
            // 今天一定標，往回每 7 天標一次
            label: (daily.length - 1 - i) % 7 === 0 ? d.date.slice(5).replace("-", "/") : "",
            value: d.total,
            unrecorded: !d.recorded,
            detail: d.recorded ? t("dayDetail", { date, total: money(d.total) }) : t("dayDetailEmpty", { date }),
        };
    });

    const monthlyBars: SpendBar[] = monthly.map(m => {
        const month = monthLongFmt.format(noon(m.month));
        return {
            key: m.month,
            label: monthFmt.format(noon(m.month)),
            value: m.total,
            unrecorded: m.recorded_days === 0,
            detail: m.recorded_days === 0
                ? t("monthDetailEmpty", { month })
                : t("monthDetail", {
                    month,
                    total: money(m.total),
                    days: m.recorded_days,
                    avg: money(average(m.total, m.recorded_days)),
                }),
        };
    });

    const change = monthChange(compare.this_month, compare.last_month_same_period);
    const ChangeIcon = change.diff > 0 ? TrendingUp : change.diff < 0 ? TrendingDown : Minus;

    const tiles = [
        { key: "total", value: money(overview.total) },
        { key: "days", value: t("daysValue", { n: overview.recorded_days, span: days }) },
        { key: "avg", value: money(average(overview.total, overview.recorded_days)) },
        { key: "free", value: t("freeValue", { n: overview.free_entries }) },
    ] as const;

    return (
        <div className="flex flex-col gap-6">
            <section className={`${CARD} px-4 py-3 flex flex-wrap items-end justify-between gap-x-6 gap-y-2`}>
                <div>
                    <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-1">{t("monthToDate")}</p>
                    <p className="font-semibold text-2xl tabular-nums">{money(compare.this_month)}</p>
                </div>
                <div className="text-sm text-right tabular-nums">
                    <p className="flex items-center justify-end gap-1">
                        <ChangeIcon size={16} className="text-neutral-500 dark:text-neutral-400" aria-hidden />
                        <span className="font-semibold">
                            {change.diff > 0 ? "+" : change.diff < 0 ? "−" : ""}{money(Math.abs(change.diff))}
                        </span>
                        {change.percent !== null && (
                            <span className="text-neutral-500 dark:text-neutral-400">
                                ({change.diff > 0 ? "+" : change.diff < 0 ? "−" : ""}{Math.abs(change.percent)}%)
                            </span>
                        )}
                    </p>
                    <p className="text-neutral-500 dark:text-neutral-400">
                        {t("vsLastMonth", { amount: money(compare.last_month_same_period) })}
                    </p>
                </div>
            </section>

            <section className={`${CARD} p-4 flex flex-col gap-2`}>
                <h2 className="font-semibold">{t("dailyTitle")}</h2>
                <SpendChart
                    bars={dailyBars}
                    ariaLabel={t("dailyTitle")}
                    formatValue={money}
                    reference={{ value: dailyAvg, label: t("dailyAvg", { amount: money(dailyAvg) }) }}
                    hint={t("chartHint")}
                />
                <p className="text-xs text-neutral-500 dark:text-neutral-400">{t("unrecordedLegend")}</p>
            </section>

            <section className={`${CARD} p-4 flex flex-col gap-2`}>
                <h2 className="font-semibold">{t("monthlyTitle")}</h2>
                <SpendChart bars={monthlyBars} ariaLabel={t("monthlyTitle")} formatValue={money} hint={t("chartHint")} />
            </section>

            <section className="flex flex-col gap-3" aria-labelledby="food-log-ranking">
                <div className="flex items-center justify-between gap-2 flex-wrap">
                    <h2 id="food-log-ranking" className="text-sm font-semibold text-neutral-500 dark:text-neutral-400">
                        {t("rankingTitle")}
                    </h2>
                    <Pills
                        options={RANGE_OPTIONS.map(n => ({ value: n, label: t(`range.${n}`) }))}
                        value={days}
                        onChange={changeRange}
                        disabled={isPending}
                    />
                </div>

                <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    {tiles.map(tile => (
                        <div key={tile.key} className={`${CARD} px-4 py-3`}>
                            <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-1">{t(`overview.${tile.key}`)}</p>
                            <p className="font-semibold text-lg tabular-nums">{tile.value}</p>
                        </div>
                    ))}
                </div>

                <div className="grid md:grid-cols-2 gap-3 items-start">
                    <Ranking
                        className={CARD}
                        title={t("stores")}
                        rows={stats.stores.map(s => ({ key: s.store, name: s.store, count: s.visits, total: s.total }))}
                        countText={n => t("times", { n })}
                    />
                    <Ranking
                        className={CARD}
                        title={t("items")}
                        rows={stats.items.map(s => ({
                            key: `${s.store ?? ""}\u0000${s.item}`,
                            name: s.item,
                            sub: s.store,
                            count: s.count,
                            total: s.total,
                        }))}
                        countText={n => t("times", { n })}
                    />
                </div>
            </section>

            <Toast toast={toast} />
        </div>
    );
}
