import AuditLogsClient from "./audit-logs-client";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Audit Logs",
    description: "Admin audit log viewer",
};

export default function AuditLogsPage() {
    return <AuditLogsClient />;
}
