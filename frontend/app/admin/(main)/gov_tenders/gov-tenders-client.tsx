"use client";

import { useEffect, useState } from "react";
import { ExternalLink } from "lucide-react";
import { getGovTenders, getGovTenderTypes } from "@/api/gov-tenders";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import type { GovTender } from "@/types";

const LIMIT = 50;

interface Filters {
    q: string;
    keyword: string;
    tender_type: string;
}

const defaultFilters: Filters = { q: '', keyword: '', tender_type: '' };

export default function GovTendersClient() {
    const { items: tenders, hasMore, isPending, load, loadMore } = usePagedList<GovTender>(LIMIT);
    const [filters, setFilters] = useState<Filters>(defaultFilters);
    const [types, setTypes] = useState<string[]>([]);

    useEffect(() => {
        load(page => getGovTenders({ page, per_page: LIMIT }));
        getGovTenderTypes().then(setTypes).catch(() => setTypes([]));
    }, [load]);

    function handleSearch() {
        if (isPending) return;
        load(page => getGovTenders({ ...filters, page, per_page: LIMIT }));
    }

    function handleReset() {
        setFilters(defaultFilters);
        load(page => getGovTenders({ page, per_page: LIMIT }));
    }

    const inputClass = "px-2 py-1.5 text-sm rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 focus:outline-none focus:ring-1 focus:ring-primary-400";

    return (
        <div className="w-full">
            <div className="flex flex-col gap-4">
                <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">政府標案</h1>

                {/* Filter bar */}
                <div className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">標案名稱 / 機關</label>
                        <input
                            type="text"
                            value={filters.q}
                            onChange={e => setFilters(f => ({ ...f, q: e.target.value }))}
                            onKeyDown={e => e.key === 'Enter' && handleSearch()}
                            placeholder="弱點掃描"
                            className={`${inputClass} w-48`}
                        />
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">類型</label>
                        <select
                            value={filters.tender_type}
                            onChange={e => setFilters(f => ({ ...f, tender_type: e.target.value }))}
                            className={`${inputClass} w-48`}
                        >
                            <option value="">全部</option>
                            {types.map(t => (
                                <option key={t} value={t}>{t}</option>
                            ))}
                        </select>
                    </div>
                    <div className="flex flex-col gap-1">
                        <label className="text-xs text-neutral-500 dark:text-neutral-400">追蹤關鍵字</label>
                        <input
                            type="text"
                            value={filters.keyword}
                            onChange={e => setFilters(f => ({ ...f, keyword: e.target.value }))}
                            onKeyDown={e => e.key === 'Enter' && handleSearch()}
                            placeholder="網站"
                            className={`${inputClass} w-32`}
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

                <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <div className="overflow-x-auto">
                        <AdminTable className="text-sm">
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh className="whitespace-nowrap">公告日</AdminTh>
                                    <AdminTh className="whitespace-nowrap hidden sm:table-cell">類型</AdminTh>
                                    <AdminTh className="min-w-[14rem]">標案名稱</AdminTh>
                                    <AdminTh className="whitespace-nowrap">機關</AdminTh>
                                    <AdminTh className="hidden lg:table-cell">廠商</AdminTh>
                                    <AdminTh className="whitespace-nowrap">關鍵字</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {tenders.length === 0 ? (
                                    <tr>
                                        <td colSpan={6} className="px-4 py-8 text-center text-neutral-500 dark:text-neutral-400">
                                            {isPending ? '載入中…' : '目前沒有標案資料（排程每日抓取一次）'}
                                        </td>
                                    </tr>
                                ) : (
                                    tenders.map(t => (
                                        <AdminRow key={t.id}>
                                            <AdminTd className="whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400">
                                                {t.date}
                                            </AdminTd>
                                            <AdminTd className="whitespace-nowrap text-xs hidden sm:table-cell">
                                                {t.tender_type}
                                            </AdminTd>
                                            <AdminTd className="max-w-[18rem] sm:max-w-md">
                                                <a
                                                    href={t.detail_url}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="inline-flex items-start gap-1 text-primary-700 dark:text-primary-300 hover:underline"
                                                >
                                                    {t.title}
                                                    <ExternalLink className="w-3.5 h-3.5 mt-0.5 shrink-0" />
                                                </a>
                                                {t.category && (
                                                    <div className="text-xs text-neutral-500 dark:text-neutral-400 mt-0.5">{t.category}</div>
                                                )}
                                            </AdminTd>
                                            <AdminTd className="text-xs">
                                                {t.unit_name}
                                            </AdminTd>
                                            <AdminTd className="hidden lg:table-cell text-xs">
                                                {t.companies.length > 0 ? t.companies.join('、') : '—'}
                                            </AdminTd>
                                            <AdminTd className="whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400">
                                                {t.keyword}
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
                            onClick={loadMore}
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
