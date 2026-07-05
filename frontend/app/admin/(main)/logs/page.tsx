import LogsClient from "./logs-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "Logs",
    description: "System logs viewer",
};

export default async function LogsPage() {
    await requirePermission("log:read");
    return <LogsClient />;
}
