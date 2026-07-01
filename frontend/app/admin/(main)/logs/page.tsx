import LogsClient from "./logs-client";
import type { Metadata } from "next";

export const metadata: Metadata = {
    title: "Logs",
    description: "System logs viewer",
};

export default function LogsPage() {
    return <LogsClient />;
}
