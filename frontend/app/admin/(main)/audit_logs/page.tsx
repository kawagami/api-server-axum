import { Suspense } from "react";
import AuditLogsClient from "./audit-logs-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export const metadata: Metadata = {
    title: "操作紀錄",
    description: "後台 API 操作紀錄查詢",
};

export default async function AuditLogsPage() {
    await requirePermission("audit:read");
    // AuditLogsClient 讀 useSearchParams（?from=&to=），需要 Suspense 邊界
    return (
        <Suspense fallback={<ListTableSkeleton headers={['時間', '操作者', '方法', '路徑', 'Query', '狀態']} rows={10} />}>
            <AuditLogsClient />
        </Suspense>
    );
}
