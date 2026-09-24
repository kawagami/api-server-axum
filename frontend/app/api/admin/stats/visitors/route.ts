import type { NextRequest } from "next/server";
import { getVisitorStats } from "@/api/stats";
import { adminJson, intParam } from "@/libs/admin-route";

// 後台輪詢用的讀取端點（為什麼不直接呼叫 Server Action 見 libs/admin-route.ts）。days 的 clamp 在後端
export async function GET(request: NextRequest) {
    const days = intParam(request.nextUrl.searchParams, "days") ?? 30;
    return adminJson(() => getVisitorStats(days));
}
