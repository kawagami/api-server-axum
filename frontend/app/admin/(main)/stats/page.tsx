import type { Metadata } from "next";
import { getVisitorStats } from "@/api/stats";
import VisitorStatsView from "./visitor-stats-view";

export const metadata: Metadata = {
    title: "到訪統計",
    description: "不重複到訪統計",
};

const DEFAULT_DAYS = 30;

export default async function StatsPage() {
    const initial = await getVisitorStats(DEFAULT_DAYS);

    return <VisitorStatsView initial={initial} initialDays={DEFAULT_DAYS} />;
}
