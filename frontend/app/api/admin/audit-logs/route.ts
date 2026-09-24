import type { NextRequest } from "next/server";
import { getAuditLogs } from "@/api/logs";
import { adminJson, intParam, strParam } from "@/libs/admin-route";
import type { AuditActorType, HttpMethod } from "@/types";

const METHODS: readonly HttpMethod[] = ["GET", "POST", "PUT", "PATCH", "DELETE"];
const ACTORS: readonly AuditActorType[] = ["admin", "member"];

// 後台輪詢 / 載入更多用的讀取端點（為什麼不直接呼叫 Server Action 見 libs/admin-route.ts）
export async function GET(request: NextRequest) {
    const p = request.nextUrl.searchParams;
    return adminJson(() => getAuditLogs({
        user_email: strParam(p, "user_email"),
        method: METHODS.find(m => m === p.get("method")),
        path: strParam(p, "path"),
        from: strParam(p, "from"),
        to: strParam(p, "to"),
        actor_type: ACTORS.find(a => a === p.get("actor_type")),
        page: intParam(p, "page"),
        per_page: intParam(p, "per_page"),
    }));
}
