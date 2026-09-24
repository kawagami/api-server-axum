import "client-only";

import type { GetAuditLogsParams, GetLogsParams } from "@/api/logs";
import type {
    AuditLog,
    GameOverview,
    Log,
    PaginatedResponse,
    SystemMetric,
    VisitorStats,
    WsConnection,
} from "@/types";

/**
 * 後台輪詢用的讀取函式（client 端）。名稱與簽名刻意和 `api/*.ts` 的同名 Server Action 一致，
 * 元件只換 import 來源；背後打的是 `app/api/admin/**` 的 GET Route Handler。
 *
 * 為什麼不直接用 Server Action：Next 在 client 端一次只送一個 Server Action，輪詢會和使用者
 * 操作互相排隊（詳見 `libs/admin-route.ts`）。**只放「會被輪詢或連續觸發的讀取」**，寫入照舊走 Server Action。
 */

type Params = Record<string, string | number | undefined | null>;

async function adminGet<T>(path: string, params: Params = {}): Promise<T> {
    const qs = new URLSearchParams();
    for (const [k, v] of Object.entries(params)) {
        if (v !== undefined && v !== null && v !== "") qs.set(k, String(v));
    }
    const url = qs.size ? `${path}?${qs}` : path;

    // manual：登入失效時 route 回 307 → /admin/login。跟隨它只會白抓一份登入頁 HTML，
    // 這裡改成自己導頁（與 adminRequest 的 401 行為一致：帶 redirect 回原頁）。
    const res = await fetch(url, { cache: "no-store", redirect: "manual" });
    if (res.type === "opaqueredirect") {
        window.location.assign(`/admin/login?redirect=${encodeURIComponent(window.location.pathname)}`);
        // 導頁中：不 resolve 也不 reject，呼叫端不會閃一下「載入失敗」
        return new Promise<T>(() => {});
    }

    const body = await res.json().catch(() => null);
    if (!res.ok) {
        // 還原成 libs/api-error.ts 的 ApiError 形狀，呼叫端的 apiErrorStatus / apiErrorMessage 照常可用
        throw Object.assign(new Error(`API ${res.status}: ${res.statusText}`), {
            status: res.status,
            errorData: body?.errorData ?? null,
        });
    }
    return body as T;
}

export function getGamesOverview(): Promise<GameOverview[]> {
    return adminGet("/api/admin/games");
}

export function getWsConnections(): Promise<WsConnection[]> {
    return adminGet("/api/admin/ws/connections");
}

export function getSystemMetrics(hours?: number): Promise<SystemMetric[]> {
    return adminGet("/api/admin/metrics", { hours });
}

export function getVisitorStats(days: number): Promise<VisitorStats> {
    return adminGet("/api/admin/stats/visitors", { days });
}

export async function getLogs(params: GetLogsParams = {}): Promise<PaginatedResponse<Log>> {
    return (await adminGet<PaginatedResponse<Log> | null>("/api/admin/logs", { ...params })) ?? { data: [], total: 0 };
}

export async function getAuditLogs(params: GetAuditLogsParams = {}): Promise<PaginatedResponse<AuditLog>> {
    return (await adminGet<PaginatedResponse<AuditLog> | null>("/api/admin/audit-logs", { ...params })) ?? { data: [], total: 0 };
}
