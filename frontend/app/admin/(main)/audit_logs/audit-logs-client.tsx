"use client";

import { useState, useEffect, useRef } from "react";
import { getAuditLogs } from "@/api/logs";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import type { AuditLog, HttpMethod } from "@/types";
import { METHOD_BADGE, httpStatusBadgeClass } from "@/libs/badge-styles";
import { formatDateTimeSeconds } from "@/libs/admin-datetime";

const LIMIT = 100;

interface Filters {
    user_email: string;
    method: HttpMethod | '';
    path: string;
    from: string;
    to: string;
}

const defaultFilters: Filters = { user_email: '', method: '', path: '', from: '', to: '' };

// datetime-local 是無時區的本地時間字串（2026-07-27T10:30），後端要 RFC3339；
// 必須在瀏覽器轉才吃得到使用者當地時區，不能丟給 server action 換算
function toIso(local: string): string | undefined {
    if (!local) return undefined;
    const t = new Date(local).getTime();
    return Number.isNaN(t) ? undefined : new Date(t).toISOString();
}

function toQuery(f: Filters) {
    return { ...f, from: toIso(f.from), to: toIso(f.to) };
}

// 反向：ISO → datetime-local 輸入框吃的本地時間字串（給 ?from=&to= 帶入用，如 /admin/metrics 選區跳來）
function toLocalInput(iso: string | null | undefined): string {
    if (!iso) return '';
    const d = new Date(iso);
    if (Number.isNaN(d.getTime())) return '';
    const p = (n: number) => String(n).padStart(2, '0');
    return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

export default function AuditLogsClient() {
    const { items: logs, setItems: setLogs, hasMore, isPending, failed, load, loadMore } = usePagedList<AuditLog>(LIMIT);
    const { initial, write } = useFilterUrl(defaultFilters);
    // 條件全部可從 URL 帶入（供 /admin/metrics 選區跳轉、重新整理、貼連結）；只在首次渲染讀一次。
    // from / to 在 URL 上是 ISO，輸入框吃的是 datetime-local，兩邊各自轉換
    const [filters, setFilters] = useState<Filters>(() => ({
        ...initial,
        from: toLocalInput(initial.from),
        to: toLocalInput(initial.to),
    }));
    const [appliedFilters, setAppliedFilters] = useState<Filters>(filters);

    /** 寫回 URL：from / to 一律存 ISO（與 metrics 面板產生的連結同一種格式） */
    const writeUrl = (f: Filters) =>
        write({ ...f, from: toIso(f.from) ?? '', to: toIso(f.to) ?? '' });

    const logsRef = useRef<AuditLog[]>([]);
    const appliedFiltersRef = useRef<Filters>(filters);

    useEffect(() => { logsRef.current = logs; }, [logs]);
    useEffect(() => { appliedFiltersRef.current = appliedFilters; }, [appliedFilters]);

    useEffect(() => {
        const initial = toQuery(appliedFiltersRef.current);
        load(page => getAuditLogs({ ...initial, page, per_page: LIMIT }));
    }, [load]);

    useEffect(() => {
        const id = setInterval(async () => {
            try {
                const fresh = await getAuditLogs({ ...toQuery(appliedFiltersRef.current), page: 1, per_page: LIMIT });
                const existingIds = new Set(logsRef.current.map(l => l.id));
                const newEntries = fresh.filter(l => !existingIds.has(l.id));
                if (newEntries.length > 0) {
                    setLogs(prev => [...newEntries, ...prev]);
                }
            } catch { /* silent */ }
        }, 1_800_000);
        return () => clearInterval(id);
    }, [setLogs]);

    function handleSearch() {
        if (isPending) return;
        setAppliedFilters(filters);
        writeUrl(filters);
        const query = toQuery(filters);
        load(page => getAuditLogs({ ...query, page, per_page: LIMIT }));
    }

    function handleReset() {
        setFilters(defaultFilters);
        setAppliedFilters(defaultFilters);
        writeUrl(defaultFilters);
        load(page => getAuditLogs({ page, per_page: LIMIT }));
    }

    function handleLoadMore() {
        if (isPending) return;
        loadMore();
    }

    const inputClass = "px-2 py-1.5 text-sm rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

    return (
        <div className="w-full">
            <div className="flex flex-col gap-4">
                <PageHeader title="操作紀錄" description="後台 API 的寫入與讀取紀錄" />

                {/* Filter bar */}
                <div className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">管理員 Email</label>
                        <input
                            type="text"
                            value={filters.user_email}
                            onChange={e => setFilters(f => ({ ...f, user_email: e.target.value }))}
                            onKeyDown={e => e.key === 'Enter' && handleSearch()}
                            placeholder="admin@example.com"
                            className={`${inputClass} w-48`}
                        />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">方法</label>
                        <select
                            value={filters.method}
                            onChange={e => setFilters(f => ({ ...f, method: e.target.value as HttpMethod | '' }))}
                            className={inputClass}
                        >
                            <option value="">全部</option>
                            {(['GET', 'POST', 'PUT', 'PATCH', 'DELETE'] as HttpMethod[]).map(m => (
                                <option key={m} value={m}>{m}</option>
                            ))}
                        </select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">路徑</label>
                        <input
                            type="text"
                            value={filters.path}
                            onChange={e => setFilters(f => ({ ...f, path: e.target.value }))}
                            onKeyDown={e => e.key === 'Enter' && handleSearch()}
                            placeholder="/images"
                            className={`${inputClass} w-36`}
                        />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">起始時間</label>
                        <input
                            type="datetime-local"
                            value={filters.from}
                            onChange={e => setFilters(f => ({ ...f, from: e.target.value }))}
                            className={inputClass}
                        />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">結束時間</label>
                        <input
                            type="datetime-local"
                            value={filters.to}
                            onChange={e => setFilters(f => ({ ...f, to: e.target.value }))}
                            className={inputClass}
                        />
                    </div>
                    <div className="flex gap-2">
                        <button
                            onClick={handleSearch}
                            disabled={isPending}
                            className="px-4 py-1.5 text-sm font-medium rounded bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors"
                        >
                            搜尋
                        </button>
                        <button
                            onClick={handleReset}
                            disabled={isPending}
                            className="px-4 py-1.5 text-sm font-medium rounded bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                        >
                            重設
                        </button>
                    </div>
                </div>

                <ErrorBanner message={failed ? LOAD_FAILED : null} />

                <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <div className="admin-sticky-head overflow-auto max-h-[70svh]">
                        <AdminTable className="text-sm">
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh className="w-32 md:w-44">時間</AdminTh>
                                    <AdminTh className="hidden md:table-cell">管理員</AdminTh>
                                    <AdminTh className="w-20">方法</AdminTh>
                                    <AdminTh>路徑</AdminTh>
                                    <AdminTh className="hidden lg:table-cell">Query</AdminTh>
                                    <AdminTh className="w-20">狀態</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {logs.length === 0 ? (
                                    <AdminEmptyRow colSpan={6}>
                                        {isPending ? '載入中…' : '目前沒有符合條件的紀錄'}
                                    </AdminEmptyRow>
                                ) : (
                                    logs.map(log => (
                                        <AdminRow key={log.id}>
                                            <AdminTd className="text-neutral-500 dark:text-neutral-400 text-xs whitespace-nowrap">
                                                {formatDateTimeSeconds(log.created_at)}
                                            </AdminTd>
                                            <AdminTd className="text-xs font-mono hidden md:table-cell">
                                                {log.user_email}
                                            </AdminTd>
                                            <AdminTd>
                                                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${METHOD_BADGE[log.method] ?? 'bg-neutral-100 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-400'}`}>
                                                    {log.method}
                                                </span>
                                            </AdminTd>
                                            <AdminTd className="font-mono text-xs break-all">
                                                {log.path}
                                            </AdminTd>
                                            <AdminTd className="text-neutral-500 dark:text-neutral-400 font-mono text-xs hidden lg:table-cell">
                                                {log.query ?? '—'}
                                            </AdminTd>
                                            <AdminTd>
                                                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${httpStatusBadgeClass(log.status_code)}`}>
                                                    {log.status_code}
                                                </span>
                                            </AdminTd>
                                        </AdminRow>
                                    ))
                                )}
                            </tbody>
                        </AdminTable>
                    </div>
                </div>

                {hasMore && (
                    <div className="flex justify-center pb-4">
                        <button
                            onClick={handleLoadMore}
                            disabled={isPending}
                            className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                        >
                            {isPending ? '載入中…' : '載入更多'}
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}
