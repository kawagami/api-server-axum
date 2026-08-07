"use client";

import { useEffect } from "react";
import Link from "next/link";
import { ExternalLink, X } from "lucide-react";
import { getAuditLogs } from "@/api/logs";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import usePagedList from "@/hooks/usePagedList";
import { METHOD_BADGE, httpStatusBadgeClass } from "@/libs/badge-styles";
import { ADMIN_LOCALE, ADMIN_TIME_ZONE } from "@/libs/admin-datetime";
import type { AuditLog } from "@/types";

const LIMIT = 50;

// 面板空間窄，省略年份；locale / 時區沿用 admin-datetime 的同一組常數
const fmtRange = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    timeZone: ADMIN_TIME_ZONE,
    month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", hourCycle: "h23",
});

const fmtRow = new Intl.DateTimeFormat(ADMIN_LOCALE, {
    timeZone: ADMIN_TIME_ZONE,
    month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", second: "2-digit", hourCycle: "h23",
});

/**
 * 圖表選取區間對應的後台操作紀錄。
 * `from` / `to` 是採樣點的 ISO 時間（UTC），直接當 RFC3339 傳給後端，
 * 與 /admin/audit_logs 打同一支 API。
 */
export default function MetricsAuditPanel({
    from,
    to,
    onClear,
}: {
    from: string;
    to: string;
    onClear: () => void;
}) {
    const { items: logs, total, hasMore, isPending, failed, load, loadMore } = usePagedList<AuditLog>();

    useEffect(() => {
        load(page => getAuditLogs({ from, to, page, per_page: LIMIT }));
    }, [load, from, to]);

    return (
        <section className="bg-white dark:bg-neutral-900 rounded-lg shadow-sm border border-neutral-200 dark:border-neutral-700 p-5 flex flex-col gap-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
                <div className="flex flex-col gap-0.5">
                    <h2 className="font-semibold text-sm text-neutral-700 dark:text-neutral-200">
                        這段時間的後台操作紀錄
                    </h2>
                    <p className="text-xs text-neutral-400 dark:text-neutral-500 tabular-nums">
                        {fmtRange.format(new Date(from))} – {fmtRange.format(new Date(to))}
                        {/* 端點回 total，不必再用「已載入筆數 +」暗示可能還有 */}
                        {total > 0 && `・${total} 筆`}
                    </p>
                </div>
                <div className="flex items-center gap-2">
                    <Link
                        href={`/admin/audit_logs?from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`}
                        className="inline-flex items-center gap-1 px-3 py-1.5 text-sm rounded-sm bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
                    >
                        <ExternalLink size={14} />
                        在操作紀錄開啟
                    </Link>
                    <button
                        type="button"
                        onClick={onClear}
                        className="inline-flex items-center gap-1 px-3 py-1.5 text-sm rounded-sm bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
                    >
                        <X size={14} />
                        清除選取
                    </button>
                </div>
            </div>

            <ErrorBanner message={failed ? LOAD_FAILED : null} />

            <div className={`overflow-x-auto transition-opacity ${isPending ? "opacity-60" : ""}`}>
                <AdminTable className="text-sm">
                    <thead>
                        <AdminHeadRow>
                            <AdminTh className="whitespace-nowrap">時間</AdminTh>
                            <AdminTh className="whitespace-nowrap hidden md:table-cell">使用者</AdminTh>
                            <AdminTh className="whitespace-nowrap">方法</AdminTh>
                            <AdminTh className="min-w-48">路徑</AdminTh>
                            <AdminTh className="whitespace-nowrap">狀態</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {logs.length === 0 ? (
                            <AdminEmptyRow colSpan={5}>
                                {isPending ? "載入中…" : "這段時間沒有後台操作紀錄"}
                            </AdminEmptyRow>
                        ) : (
                            logs.map(log => (
                                <AdminRow key={log.id}>
                                    <AdminTd className="text-neutral-500 dark:text-neutral-400 text-xs whitespace-nowrap tabular-nums">
                                        {fmtRow.format(new Date(log.created_at))}
                                    </AdminTd>
                                    <AdminTd className="text-xs font-mono hidden md:table-cell">
                                        {log.user_email}
                                    </AdminTd>
                                    <AdminTd>
                                        <span className={`px-2 py-0.5 rounded-sm text-xs font-semibold ${METHOD_BADGE[log.method] ?? 'bg-neutral-100 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-400'}`}>
                                            {log.method}
                                        </span>
                                    </AdminTd>
                                    <AdminTd className="font-mono text-xs break-all">{log.path}</AdminTd>
                                    <AdminTd>
                                        <span className={`px-2 py-0.5 rounded-sm text-xs font-semibold ${httpStatusBadgeClass(log.status_code)}`}>
                                            {log.status_code}
                                        </span>
                                    </AdminTd>
                                </AdminRow>
                            ))
                        )}
                    </tbody>
                </AdminTable>
            </div>

            {hasMore && (
                <div className="flex justify-center">
                    <button
                        type="button"
                        onClick={() => { if (!isPending) loadMore(); }}
                        disabled={isPending}
                        className="px-5 py-1.5 text-sm font-medium rounded-sm bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 transition-colors"
                    >
                        {isPending ? "載入中…" : "載入更多"}
                    </button>
                </div>
            )}
        </section>
    );
}
