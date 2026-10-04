import { getFoodLogOverview } from "@/api/food-log";
import FoodLogClient from "./food-log-client";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { Link } from "@/i18n/navigation";
import { LayoutDashboard } from "lucide-react";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import { addDays, taipeiToday } from "./model";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("FoodLog");
    return { title: t("title") };
}

/** 首屏顯示的天數（含今天），往前翻每次再多這麼多天 */
const PAGE_DAYS = 7;

export default async function FoodLogPage() {
    const today = taipeiToday();
    const from = addDays(today, -(PAGE_DAYS - 1));
    const [overview, t, tHeader] = await Promise.all([
        getFoodLogOverview(from, today),
        getTranslations("FoodLog"),
        getTranslations("Header"),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle
                title={t("title")}
                actions={
                    <Link
                        href="/dashboard"
                        className="flex items-center gap-1 text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-500 dark:hover:text-primary-400 transition-colors"
                    >
                        <LayoutDashboard size={16} />
                        {tHeader("dashboard")}
                    </Link>
                }
            />
            <FoodLogClient initial={overview} initialFrom={from} today={today} pageDays={PAGE_DAYS} />
        </PageShell>
    );
}
