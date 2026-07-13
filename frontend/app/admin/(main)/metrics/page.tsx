import type { Metadata } from "next";
import { getSystemMetrics } from "@/api/metrics";
import MetricsView from "./metrics-view";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "系統指標",
    description: "主機 CPU / 記憶體 / 磁碟 / load 時間序列",
};

const DEFAULT_HOURS = 24;

export default async function MetricsPage() {
    await requirePermission("metric:read");
    const initial = await getSystemMetrics(DEFAULT_HOURS);

    return <MetricsView initial={initial} initialHours={DEFAULT_HOURS} />;
}
