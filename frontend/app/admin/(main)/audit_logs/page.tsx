import AuditLogsClient from "./audit-logs-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "Audit Logs",
    description: "Admin audit log viewer",
};

export default async function AuditLogsPage() {
    await requirePermission("audit:read");
    return <AuditLogsClient />;
}
