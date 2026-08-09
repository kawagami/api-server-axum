"use server";

import adminRequest from "@/libs/adminRequest";
import type { Log, LogLevel, AuditLog, AuditActorType, HttpMethod, PaginatedResponse } from "@/types";

interface GetLogsParams {
    level?: LogLevel;
    /** message 與 fields 一起模糊比對 —— 錯誤細節在 fields.self，只搜 message 找不到 */
    q?: string;
    page?: number;
    per_page?: number;
}

/** `GET /logs` 回 `{ data, total }`（2026-08-03 起，原本是裸陣列） */
export async function getLogs({
    level,
    q,
    page = 1,
    per_page = 100,
}: GetLogsParams = {}): Promise<PaginatedResponse<Log>> {
    const params = new URLSearchParams();
    if (level) params.set('level', level);
    if (q) params.set('q', q);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    const res = await adminRequest<PaginatedResponse<Log>>({
        url: `${process.env.API_URL}/logs?${params}`,
    });
    return res ?? { data: [], total: 0 };
}

/**
 * `GET /logs/request/{request_id}` —— 單一請求的完整軌跡。
 *
 * 回的是**裸陣列**且**時間正序**（與列表的新到舊刻意相反，上限 500 筆，後端刻意不分頁）。
 * 一個 5xx 通常是 3 筆同 request_id：來源處的 error ＋ errors.rs 的統一那筆 ＋
 * tower_http 的 `response failed`。
 */
export async function getLogTrace(requestId: string): Promise<Log[]> {
    const res = await adminRequest<Log[]>({
        url: `${process.env.API_URL}/logs/request/${encodeURIComponent(requestId)}`,
    });
    return res ?? [];
}

export interface GetAuditLogsParams {
    user_email?: string;
    method?: HttpMethod | '';
    path?: string;
    from?: string;
    to?: string;
    /** 不給 = admin + member 都列 */
    actor_type?: AuditActorType | '';
    page?: number;
    per_page?: number;
}

/** `GET /admin/audit_logs` 回 `{ data, total }`（與 `/logs` 同形） */
export async function getAuditLogs({
    user_email,
    method,
    path,
    from,
    to,
    actor_type,
    page = 1,
    per_page = 100,
}: GetAuditLogsParams = {}): Promise<PaginatedResponse<AuditLog>> {
    const params = new URLSearchParams();
    if (user_email) params.set('user_email', user_email);
    if (method) params.set('method', method);
    if (path) params.set('path', path);
    if (actor_type) params.set('actor_type', actor_type);
    if (from) params.set('from', from);
    if (to) params.set('to', to);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    const res = await adminRequest<PaginatedResponse<AuditLog>>({
        url: `${process.env.API_URL}/admin/audit_logs?${params}`,
    });
    return res ?? { data: [], total: 0 };
}
