import { getWsConnections } from "@/api/ws";
import WsConnections from "./ws-connections";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "WebSocket 連線",
    description: "即時線上連線與訊息發送",
};

export default async function WsAdminPage() {
    await requirePermission("ws:read");
    const initial = await getWsConnections();

    return (
        <div className="w-full">
            <WsConnections initial={initial} />
        </div>
    );
}
