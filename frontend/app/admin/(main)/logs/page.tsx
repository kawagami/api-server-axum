import { Suspense } from "react";
import LogsClient from "./logs-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";

const SKELETON_HEADERS = ['ID', '層級', '訊息', '來源模組', '檔案', '時間'];

export const metadata: Metadata = {
    title: "系統日誌",
    description: "後端執行日誌（INFO / WARN / ERROR）",
};

export default async function LogsPage() {
    await requirePermission("log:read");
    // LogsClient 讀 useSearchParams（?level=）當初始條件，需要 Suspense 邊界
    return (
        <Suspense fallback={<ListTableSkeleton headers={SKELETON_HEADERS} rows={10} />}>
            <LogsClient />
        </Suspense>
    );
}
