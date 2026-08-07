"use client";

import { useState, useEffect } from "react";
import { getLogs } from "@/api/logs";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import type { Log, LogLevel } from "@/types";
import { LEVEL_BADGE, LEVEL_ROW_BG } from "@/libs/badge-styles";
import { formatDateTimeSeconds } from "@/libs/admin-datetime";

const LIMIT = 100;

type LevelFilter = '' | LogLevel;

const LEVEL_FILTERS: { value: LevelFilter; label: string }[] = [
    { value: '', label: '全部' },
    { value: 'INFO', label: 'INFO' },
    { value: 'WARN', label: 'WARN' },
    { value: 'ERROR', label: 'ERROR' },
];

const VALID_LEVELS: LevelFilter[] = LEVEL_FILTERS.map(f => f.value);
const defaultFilters = { level: '' as string };

export default function LogsClient() {
    const { items: logs, hasMore, isPending, failed, load, loadMore } = usePagedList<Log>();
    const { initial, write } = useFilterUrl(defaultFilters);
    // URL 是使用者可以亂打的，不在白名單內的 level 一律當成「全部」
    const [level, setLevel] = useState<LevelFilter>(
        () => (VALID_LEVELS.includes(initial.level as LevelFilter) ? initial.level as LevelFilter : '')
    );

    useEffect(() => {
        load(page => getLogs({ level: level || undefined, page, per_page: LIMIT }));
        // 初次載入沿用 URL 帶進來的條件；後續改條件走 handleFilterChange
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [load]);

    function handleFilterChange(newLevel: LevelFilter) {
        if (newLevel === level || isPending) return;
        setLevel(newLevel);
        write({ level: newLevel });
        load(page => getLogs({ level: newLevel || undefined, page, per_page: LIMIT }));
    }

    function handleLoadMore() {
        if (isPending) return;
        loadMore();
    }

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
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {logs.length === 0 ? (
                                    <AdminEmptyRow colSpan={6}>
                                        {isPending ? '載入中…' : '目前沒有日誌'}
                                    </AdminEmptyRow>
                                ) : (
                                    logs.map((log) => (
                                        <AdminRow key={log.id} tone={LEVEL_ROW_BG[log.level]}>
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
