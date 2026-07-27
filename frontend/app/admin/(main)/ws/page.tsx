import { getWsConnections } from "@/api/ws";
import WsConnections from "./ws-connections";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "WS 連線管理",
    description: "即時線上 WebSocket 連線與訊息發送",
};

export default async function WsAdminPage() {
    await requirePermission("ws:read");
    const initial = await getWsConnections();

    return (
        <div className="w-full">
            <div className="max-w-5xl mx-auto">
                <WsConnections initial={initial} />
            </div>
        </div>
    );
}
