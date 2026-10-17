import { getFoodLogStats } from "@/api/food-log";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { Link } from "@/i18n/navigation";
import { Utensils } from "lucide-react";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import StatsClient from "./stats-client";
import { DEFAULT_RANGE } from "./stats-model";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("FoodLogStats");
    return { title: t("title") };
}

export default async function FoodLogStatsPage() {
    const [result, t, tLog] = await Promise.all([
        getFoodLogStats(DEFAULT_RANGE),
        getTranslations("FoodLogStats"),
        getTranslations("FoodLog"),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle
                title={t("title")}
                actions={
                    <Link
                        href="/food-log"
                        className="flex items-center gap-1 text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-500 dark:hover:text-primary-400 transition-colors"
                    >
                        <Utensils size={16} />
                        {tLog("title")}
                    </Link>
                }
            />
            {result.ok ? (
                <StatsClient initial={result.data} initialDays={DEFAULT_RANGE} />
            ) : (
                <p className="text-center text-neutral-500 dark:text-neutral-400 py-12">{t("loadFailed")}</p>
            )}
        </PageShell>
    );
}
