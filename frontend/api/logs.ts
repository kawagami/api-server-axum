"use server";

import adminRequest from "@/libs/adminRequest";
import type { Log, LogLevel, AuditLog, HttpMethod } from "@/types";

interface GetLogsParams {
    level?: LogLevel;
    page?: number;
    per_page?: number;
}

export async function getLogs({ level, page = 1, per_page = 100 }: GetLogsParams = {}): Promise<Log[]> {
    const params = new URLSearchParams();
    if (level) params.set('level', level);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    return adminRequest<Log[]>({
        url: `${process.env.API_URL}/logs?${params}`,
    });
}

export interface GetAuditLogsParams {
    user_email?: string;
    method?: HttpMethod | '';
    path?: string;
    from?: string;
    to?: string;
    page?: number;
    per_page?: number;
}

export async function getAuditLogs({
    user_email,
    method,
    path,
    from,
    to,
    page = 1,
    per_page = 100,
}: GetAuditLogsParams = {}): Promise<AuditLog[]> {
    const params = new URLSearchParams();
    if (user_email) params.set('user_email', user_email);
    if (method) params.set('method', method);
    if (path) params.set('path', path);
    if (from) params.set('from', from);
    if (to) params.set('to', to);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    return adminRequest<AuditLog[]>({
        url: `${process.env.API_URL}/admin/audit_logs?${params}`,
    });
}
