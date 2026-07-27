import { Suspense } from "react";
import AuditLogsClient from "./audit-logs-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";

export const metadata: Metadata = {
    title: "Audit Logs",
    description: "Admin audit log viewer",
};

export default async function AuditLogsPage() {
    await requirePermission("audit:read");
    // AuditLogsClient 讀 useSearchParams（?from=&to=），需要 Suspense 邊界
    return (
        <Suspense fallback={<ListTableSkeleton headers={['Time', 'User', 'Method', 'Path', 'Query', 'Status']} rows={10} />}>
            <AuditLogsClient />
        </Suspense>
    );
}
