"use client";

import { Fragment, useState, useEffect } from "react";
import { getLogs, getLogTrace } from "@/api/logs";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import type { Log, LogLevel } from "@/types";
import { LEVEL_BADGE, LEVEL_ROW_BG } from "@/libs/badge-styles";
import { formatDateTimeSeconds } from "@/libs/admin-datetime";

const LIMIT = 100;
const COLUMNS = 7;

type LevelFilter = '' | LogLevel;

const LEVEL_FILTERS: { value: LevelFilter; label: string }[] = [
    { value: '', label: '全部' },
    { value: 'INFO', label: 'INFO' },
    { value: 'WARN', label: 'WARN' },
    { value: 'ERROR', label: 'ERROR' },
];

const VALID_LEVELS: LevelFilter[] = LEVEL_FILTERS.map(f => f.value);
const defaultFilters = { level: '', q: '' };

/**
 * fields 的顯示順序。`self` 一定排第一 —— 那是真正的錯誤原因
 * （message 只是 `System error occurred` 這種固定字串），其餘照請求上下文的閱讀順序。
 * 不在清單裡的 key 依字母序接在後面，所以新增 span field 不必改這裡。
 */
const FIELD_ORDER = ['self', 'panic', 'method', 'path', 'query', 'ip', 'status', 'latency_ms'];

function sortedFields(fields: Record<string, unknown>): [string, string][] {
    return Object.entries(fields)
        .map(([k, v]): [string, string] => [k, typeof v === 'string' ? v : JSON.stringify(v)])
        .sort(([a], [b]) => {
            const ia = FIELD_ORDER.indexOf(a);
            const ib = FIELD_ORDER.indexOf(b);
            if (ia !== -1 && ib !== -1) return ia - ib;
            if (ia !== -1) return -1;
            if (ib !== -1) return 1;
            return a.localeCompare(b);
        });
}

export default function LogsClient() {
    const { items: logs, hasMore, isPending, failed, load, loadMore } = usePagedList<Log>();
    const { initial, write } = useFilterUrl(defaultFilters);
    // URL 是使用者可以亂打的，不在白名單內的 level 一律當成「全部」
    const [level, setLevel] = useState<LevelFilter>(
        () => (VALID_LEVELS.includes(initial.level as LevelFilter) ? initial.level as LevelFilter : '')
    );
    const [q, setQ] = useState(initial.q ?? '');
    const [appliedQ, setAppliedQ] = useState(initial.q ?? '');
    const [expandedId, setExpandedId] = useState<number | null>(null);
    // 同一個 request_id 的完整軌跡（時間正序）。只快取最近查的那一筆就夠用
    const [trace, setTrace] = useState<{ requestId: string; rows: Log[] } | null>(null);
    const [tracePending, setTracePending] = useState(false);

    useEffect(() => {
        load(page => getLogs({ level: level || undefined, q: appliedQ || undefined, page, per_page: LIMIT }));
        // 初次載入沿用 URL 帶進來的條件；後續改條件走 handleFilterChange / handleSearch
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [load]);

    function reload(nextLevel: LevelFilter, nextQ: string) {
        write({ level: nextLevel, q: nextQ });
        load(page => getLogs({ level: nextLevel || undefined, q: nextQ || undefined, page, per_page: LIMIT }));
    }

    function handleFilterChange(newLevel: LevelFilter) {
        if (newLevel === level || isPending) return;
        setLevel(newLevel);
        reload(newLevel, appliedQ);
    }

    function handleSearch() {
        if (isPending) return;
        setAppliedQ(q);
        reload(level, q);
    }

    function handleLoadMore() {
        if (isPending) return;
        loadMore();
    }

    function toggleExpand(log: Log) {
        setExpandedId(prev => (prev === log.id ? null : log.id));
    }

    async function showTrace(requestId: string) {
        if (tracePending) return;
        setTracePending(true);
        try {
            setTrace({ requestId, rows: await getLogTrace(requestId) });
        } catch {
            setTrace({ requestId, rows: [] });
        } finally {
            setTracePending(false);
        }
    }

    const inputClass = "px-2 py-1.5 text-sm rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

    return (
        <div className="w-full">
            <div className="flex flex-col gap-4">
                <PageHeader
                    title="系統日誌"
                    actions={LEVEL_FILTERS.map(({ value, label }) => (
                        <button
                            key={value || 'ALL'}
                            onClick={() => handleFilterChange(value)}
                            disabled={isPending}
                            className={`px-3 py-1 rounded text-sm font-medium transition-colors disabled:opacity-50 ${
                                level === value
                                    ? 'bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900'
                                    : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200 dark:hover:bg-neutral-700'
                            }`}
                        >
                            {label}
                        </button>
                    ))}
                />

                {/* 搜尋 message 與 fields —— 錯誤細節在 fields.self，只搜 message 找不到有用的東西 */}
                <div className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                    <div className="flex flex-col gap-1 grow">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">
                            關鍵字（同時比對訊息與錯誤細節）
                        </label>
                        <input
                            type="text"
                            value={q}
                            onChange={e => setQ(e.target.value)}
                            onKeyDown={e => e.key === 'Enter' && handleSearch()}
                            placeholder="Cannot assign requested address"
                            className={`${inputClass} w-full`}
                        />
                    </div>
                    <button
                        onClick={handleSearch}
                        disabled={isPending}
                        className="px-4 py-1.5 text-sm font-medium rounded-sm bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors"
                    >
                        搜尋
                    </button>
                </div>

                <ErrorBanner message={failed ? LOAD_FAILED : null} />

                <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <div className="admin-sticky-head overflow-auto max-h-[70svh]">
                        <AdminTable className="text-sm">
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh className="w-16 hidden sm:table-cell">ID</AdminTh>
                                    <AdminTh className="w-20">層級</AdminTh>
                                    <AdminTh>訊息</AdminTh>
                                    <AdminTh className="hidden lg:table-cell">來源模組</AdminTh>
                                    <AdminTh className="hidden xl:table-cell">檔案</AdminTh>
                                    <AdminTh className="w-32 md:w-44">時間</AdminTh>
                                    <AdminTh className="w-20">細節</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {logs.length === 0 ? (
                                    <AdminEmptyRow colSpan={COLUMNS}>
                                        {isPending ? '載入中…' : '目前沒有日誌'}
                                    </AdminEmptyRow>
                                ) : (
                                    logs.map((log) => {
                                        const fields = log.fields ?? {};
                                        const hasDetail = Object.keys(fields).length > 0 || !!log.request_id;
                                        const expanded = expandedId === log.id;
                                        return (
                                            <Fragment key={log.id}>
                                                <AdminRow tone={LEVEL_ROW_BG[log.level]}>
                                                    <AdminTd className="text-neutral-500 dark:text-neutral-500 font-mono hidden sm:table-cell">{log.id}</AdminTd>
                                                    <AdminTd>
                                                        <span className={`px-2 py-0.5 rounded-sm text-xs font-semibold ${LEVEL_BADGE[log.level]}`}>
                                                            {log.level}
                                                        </span>
                                                    </AdminTd>
                                                    <AdminTd className="font-mono break-all">{log.message}</AdminTd>
                                                    <AdminTd className="text-neutral-600 dark:text-neutral-400 font-mono text-xs hidden lg:table-cell">{log.target}</AdminTd>
                                                    <AdminTd className="text-neutral-600 dark:text-neutral-400 font-mono text-xs hidden xl:table-cell">
                                                        {log.file}:{log.line}
                                                    </AdminTd>
                                                    <AdminTd className="text-neutral-500 dark:text-neutral-400 text-xs whitespace-nowrap">
                                                        {formatDateTimeSeconds(log.created_at)}
                                                    </AdminTd>
                                                    <AdminTd>
                                                        {hasDetail ? (
                                                            <button
                                                                onClick={() => toggleExpand(log)}
                                                                className="px-2 py-0.5 rounded-sm text-xs font-medium bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
                                                            >
                                                                {expanded ? '收合' : '展開'}
                                                            </button>
                                                        ) : (
                                                            <span className="text-xs text-neutral-400 dark:text-neutral-600">—</span>
                                                        )}
                                                    </AdminTd>
                                                </AdminRow>

                                                {expanded && (
                                                    <tr>
                                                        <td
                                                            colSpan={COLUMNS}
                                                            className="border border-neutral-300 dark:border-neutral-700 bg-neutral-50 dark:bg-neutral-800/40 px-4 py-3"
                                                        >
                                                            <div className="flex flex-col gap-3">
                                                                {log.request_id && (
                                                                    <div className="flex flex-wrap items-center gap-2">
                                                                        <span className="text-xs text-neutral-500 dark:text-neutral-400">request_id</span>
                                                                        <code className="font-mono text-xs break-all">{log.request_id}</code>
                                                                        <button
                                                                            onClick={() => showTrace(log.request_id!)}
                                                                            disabled={tracePending}
                                                                            className="px-2 py-0.5 rounded-sm text-xs font-medium bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 transition-colors"
                                                                        >
                                                                            {tracePending ? '載入中…' : '整條軌跡'}
                                                                        </button>
                                                                    </div>
                                                                )}

                                                                {sortedFields(fields).map(([key, value]) => (
                                                                    <div key={key} className="flex flex-col gap-1">
                                                                        <span className="text-xs text-neutral-500 dark:text-neutral-400">{key}</span>
                                                                        <pre className="font-mono text-xs whitespace-pre-wrap break-all text-neutral-800 dark:text-neutral-200">
                                                                            {value}
                                                                        </pre>
                                                                    </div>
                                                                ))}

                                                                {trace && trace.requestId === log.request_id && (
                                                                    <div className="flex flex-col gap-1 border-t border-neutral-300 dark:border-neutral-700 pt-3">
                                                                        <span className="text-xs text-neutral-500 dark:text-neutral-400">
                                                                            同一請求的完整軌跡（時間正序，{trace.rows.length} 筆）
                                                                        </span>
                                                                        {trace.rows.length === 0 ? (
                                                                            <span className="text-xs text-neutral-400 dark:text-neutral-600">查不到紀錄</span>
                                                                        ) : (
                                                                            trace.rows.map(row => (
                                                                                <div key={row.id} className="flex flex-wrap items-baseline gap-2 font-mono text-xs">
                                                                                    <span className="text-neutral-500 dark:text-neutral-400">
                                                                                        {formatDateTimeSeconds(row.created_at)}
                                                                                    </span>
                                                                                    <span className={`px-1.5 rounded-sm ${LEVEL_BADGE[row.level]}`}>{row.level}</span>
                                                                                    <span className="text-neutral-600 dark:text-neutral-400">{row.target}</span>
                                                                                    <span className="break-all">{row.message}</span>
                                                                                </div>
                                                                            ))
                                                                        )}
                                                                    </div>
                                                                )}
                                                            </div>
                                                        </td>
                                                    </tr>
                                                )}
                                            </Fragment>
                                        );
                                    })
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
                            className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded-sm hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                        >
                            {isPending ? '載入中…' : '載入更多'}
                        </button>
                    </div>
                )}
            </div>
        </div>
    );
}
