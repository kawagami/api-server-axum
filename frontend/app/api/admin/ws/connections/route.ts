import { getWsConnections } from "@/api/ws";
import { adminJson } from "@/libs/admin-route";

// 後台輪詢用的讀取端點（為什麼不直接呼叫 Server Action 見 libs/admin-route.ts）
export async function GET() {
    return adminJson(() => getWsConnections());
}
