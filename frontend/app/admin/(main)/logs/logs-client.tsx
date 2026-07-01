"use client";

import { useState, useEffect } from "react";
import { getLogs } from "@/api/logs";
import ErrorBanner from "@/components/admin/error-banner";
import usePagedList from "@/hooks/usePagedList";
import type { Log, LogLevel } from "@/types";
import { LEVEL_BADGE, LEVEL_ROW_BG } from "@/libs/badge-styles";

const LIMIT = 100;

type LevelFilter = '' | LogLevel;

export default function LogsClient() {
    const { items: logs, hasMore, isPending, load, loadMore } = usePagedList<Log>(LIMIT);
    const [level, setLevel] = useState<LevelFilter>('');
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        load(page => getLogs({ page, per_page: LIMIT }));
    }, [load]);

    function handleFilterChange(newLevel: LevelFilter) {
        if (newLevel === level || isPending) return;
        setError(null);
        setLevel(newLevel);
        load(page => getLogs({ level: newLevel || undefined, page, per_page: LIMIT }));
    }

    function handleLoadMore() {
        if (isPending) return;
        setError(null);
        loadMore();
    }

    return (
        <div className="w-full">
            <div className="flex flex-col gap-4">
                <div className="flex items-center justify-between flex-wrap gap-3">
                    <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">
                        Logs
                    </h1>
                    <div className="flex gap-2">
                        {(['', 'INFO', 'WARN', 'ERROR'] as LevelFilter[]).map((lv) => (
                            <button
                                key={lv || 'ALL'}
                                onClick={() => handleFilterChange(lv)}
                                disabled={isPending}
                                className={`px-3 py-1 rounded text-sm font-medium transition-colors disabled:opacity-50 ${
                                    level === lv
                                        ? 'bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900'
                                        : 'bg-neutral-100 dark:bg-neutral-800 text-neutral-600 dark:text-neutral-400 hover:bg-neutral-200 dark:hover:bg-neutral-700'
                                }`}
                            >
                                {lv || 'ALL'}
                            </button>
                        ))}
                    </div>
                </div>

                <ErrorBanner message={error} />

                <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <div className="overflow-x-auto">
                    <table className="w-full border-collapse text-sm">
                        <thead>
                            <tr className="bg-neutral-100 dark:bg-neutral-800">
                                <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 w-16">ID</th>
                                <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 w-20">Level</th>
                                <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700">Message</th>
                                <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 hidden lg:table-cell">Target</th>
                                <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 hidden xl:table-cell">File</th>
                                <th className="px-4 py-2 text-left text-neutral-700 dark:text-neutral-300 border-b border-neutral-200 dark:border-neutral-700 w-44">Time</th>
                            </tr>
                        </thead>
                        <tbody>
                            {logs.length === 0 ? (
                                <tr>
                                    <td colSpan={6} className="px-4 py-8 text-center text-neutral-500 dark:text-neutral-400">
                                        {isPending ? 'Loading...' : 'No logs found'}
                                    </td>
                                </tr>
                            ) : (
                                logs.map((log) => (
                                    <tr
                                        key={log.id}
                                        className={`border-b border-neutral-100 dark:border-neutral-800 ${LEVEL_ROW_BG[log.level]}`}
                                    >
                                        <td className="px-4 py-2 text-neutral-500 dark:text-neutral-500 font-mono">{log.id}</td>
                                        <td className="px-4 py-2">
                                            <span className={`px-2 py-0.5 rounded text-xs font-semibold ${LEVEL_BADGE[log.level]}`}>
                                                {log.level}
                                            </span>
                                        </td>
                                        <td className="px-4 py-2 text-neutral-900 dark:text-neutral-100 font-mono break-all">{log.message}</td>
                                        <td className="px-4 py-2 text-neutral-600 dark:text-neutral-400 font-mono text-xs hidden lg:table-cell">{log.target}</td>
                                        <td className="px-4 py-2 text-neutral-600 dark:text-neutral-400 font-mono text-xs hidden xl:table-cell">
                                            {log.file}:{log.line}
                                        </td>
                                        <td className="px-4 py-2 text-neutral-500 dark:text-neutral-400 text-xs whitespace-nowrap">
                                            {new Date(log.created_at).toLocaleString()}
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
