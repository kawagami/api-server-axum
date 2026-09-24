import type { NextRequest } from "next/server";
import { getLogs } from "@/api/logs";
import { adminJson, intParam, strParam } from "@/libs/admin-route";
import type { LogLevel } from "@/types";

const LEVELS: readonly LogLevel[] = ["INFO", "WARN", "ERROR"];

// 後台輪詢 / 載入更多用的讀取端點（為什麼不直接呼叫 Server Action 見 libs/admin-route.ts）
export async function GET(request: NextRequest) {
    const p = request.nextUrl.searchParams;
    const level = LEVELS.find(l => l === p.get("level"));
    return adminJson(() => getLogs({
        level,
        q: strParam(p, "q"),
        page: intParam(p, "page"),
        per_page: intParam(p, "per_page"),
    }));
}
