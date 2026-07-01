"use client";

import { useState, useEffect, useRef } from "react";
import { getAuditLogs } from "@/api/logs";
import ErrorBanner from "@/components/admin/error-banner";
import usePagedList from "@/hooks/usePagedList";
import type { AuditLog, HttpMethod } from "@/types";
import { METHOD_BADGE, httpStatusBadgeClass } from "@/libs/badge-styles";

const LIMIT = 100;

interface Filters {
    user_email: string;
    method: HttpMethod | '';
    path: string;
    from: string;
    to: string;
}

const defaultFilters: Filters = { user_email: '', method: '', path: '', from: '', to: '' };

export default function AuditLogsClient() {
    const { items: logs, setItems: setLogs, hasMore, isPending, load, loadMore } = usePagedList<AuditLog>(LIMIT);
    const [filters, setFilters] = useState<Filters>(defaultFilters);
    const [appliedFilters, setAppliedFilters] = useState<Filters>(defaultFilters);
    const [error, setError] = useState<string | null>(null);

    const logsRef = useRef<AuditLog[]>([]);
    const appliedFiltersRef = useRef<Filters>(defaultFilters);

    useEffect(() => { logsRef.current = logs; }, [logs]);
    useEffect(() => { appliedFiltersRef.current = appliedFilters; }, [appliedFilters]);

    useEffect(() => {
        load(page => getAuditLogs({ page, per_page: LIMIT }));
    }, [load]);

    useEffect(() => {
        const id = setInterval(async () => {
            try {
                const fresh = await getAuditLogs({ ...appliedFiltersRef.current, page: 1, per_page: LIMIT });
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
        setError(null);
        setAppliedFilters(filters);
        load(page => getAuditLogs({ ...filters, page, per_page: LIMIT }));
    }

    function handleReset() {
        setFilters(defaultFilters);
        setAppliedFilters(defaultFilters);
        setError(null);
        load(page => getAuditLogs({ page, per_page: LIMIT }));
    }

    function handleLoadMore() {
        if (isPending) return;
        loadMore();
    }

    const inputClass = "px-2 py-1.5 text-sm rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-1 focus:ring-primary-400";

    return (
        <div className="w-full">
            <div className="flex flex-col gap-4">
                <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">Audit Logs</h1>

                {/* Filter bar */}
                <div className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">User email</label>
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
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">Method</label>
                        <select
                            value={filters.method}
                            onChange={e => setFilters(f => ({ ...f, method: e.target.value as HttpMethod | '' }))}
                            className={inputClass}
                        >
                            <option value="">ALL</option>
                            {(['GET', 'POST', 'PUT', 'PATCH', 'DELETE'] as HttpMethod[]).map(m => (
                                <option key={m} value={m}>{m}</option>
                            ))}
                        </select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">Path</label>
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
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">From</label>
                        <input
                            type="datetime-local"
                            value={filters.from}
                            onChange={e => setFilters(f => ({ ...f, from: e.target.value }))}
                            className={inputClass}
                        />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">To</label>
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
                            Search
                        </button>
                        <button
                            onClick={handleReset}
                            disabled={isPending}
                            className="px-4 py-1.5 text-sm font-medium rounded bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                        >
                            Reset
                        </button>
                    </div>
                </div>

                <ErrorBanner message={error} />

                <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <div className="overflow-x-auto">
                        <table className="w-full border-collapse text-sm">
                            <thead>
                                <tr className="bg-neutral-100 dark:bg-neutral-800">
                                    <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 w-44">Time</th>
                                    <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700">User</th>
                                    <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 w-20">Method</th>
                                    <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700">Path</th>
                                    <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 hidden lg:table-cell">Query</th>
                                    <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 w-20">Status</th>
                                </tr>
                            </thead>
                            <tbody>
                                {logs.length === 0 ? (
                                    <tr>
                                        <td colSpan={6} className="px-4 py-8 text-center text-neutral-500 dark:text-neutral-400">
                                            {isPending ? 'Loading...' : 'No audit logs found'}
                                        </td>
                                    </tr>
                                ) : (
                                    logs.map(log => (
                                        <tr
                                            key={log.id}
                                            className="border-b border-neutral-100 dark:border-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-800/50"
                                        >
                                            <td className="px-4 py-2 text-neutral-500 dark:text-neutral-400 text-xs whitespace-nowrap">
                                                {new Date(log.created_at).toLocaleString()}
                                            </td>
                                            <td className="px-4 py-2 text-neutral-900 dark:text-neutral-100 text-xs font-mono">
                                                {log.user_email}
                                            </td>
                                            <td className="px-4 py-2">
                                                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${METHOD_BADGE[log.method] ?? 'bg-neutral-100 dark:bg-neutral-700 text-neutral-600 dark:text-neutral-400'}`}>
                                                    {log.method}
                                                </span>
                                            </td>
                                            <td className="px-4 py-2 text-neutral-900 dark:text-neutral-100 font-mono text-xs break-all">
                                                {log.path}
                                            </td>
                                            <td className="px-4 py-2 text-neutral-500 dark:text-neutral-400 font-mono text-xs hidden lg:table-cell">
                                                {log.query ?? '—'}
                                            </td>
                                            <td className="px-4 py-2">
                                                <span className={`px-2 py-0.5 rounded text-xs font-semibold ${httpStatusBadgeClass(log.status_code)}`}>
                                                    {log.status_code}
                                                </span>
                                            </td>
                                        </tr>
                                    ))
                                )}
                            </tbody>
                        </table>
                    </div>
                </div>

                {hasMore && (
                    <div className="flex justify-center pb-4">
                        <button
                            onClick={handleLoadMore}
                            disabled={isPending}
                            className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                        >
                            {isPending ? 'Loading...' : 'Load More'}
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}
