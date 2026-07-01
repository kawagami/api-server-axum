"use server";

import adminRequest from "@/libs/adminRequest";
import type { WsConnection } from "@/types";

export async function getWsConnections(): Promise<WsConnection[]> {
    return adminRequest<WsConnection[]>({
        url: `${process.env.API_URL}/ws/get_online_connections`,
    });
}
