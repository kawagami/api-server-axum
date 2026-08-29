"use client";

import { useEffect, useState } from "react";
import { ExternalLink } from "lucide-react";
import { getGovTenders, getGovTenderTypes } from "@/api/gov-tenders";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import ErrorBanner, { LOAD_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import type { GovTender } from "@/types";

const LIMIT = 50;

interface Filters {
    q: string;
    keyword: string;
    tender_type: string;
}

const defaultFilters: Filters = { q: '', keyword: '', tender_type: '' };

export default function GovTendersClient() {
    const { items: tenders, hasMore, isPending, failed, load, loadMore } = usePagedList<GovTender>();
    const { initial, write } = useFilterUrl(defaultFilters);
    const [filters, setFilters] = useState<Filters>(initial);
    const [types, setTypes] = useState<string[]>([]);

    useEffect(() => {
        // 初次載入沿用 URL 帶進來的條件（重新整理 / 貼連結不掉查詢）
        load(page => getGovTenders({ ...initial, page, per_page: LIMIT }));
        getGovTenderTypes().then(setTypes).catch(() => setTypes([]));
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [load]);

    function handleSearch() {
        if (isPending) return;
        write(filters);
        load(page => getGovTenders({ ...filters, page, per_page: LIMIT }));
    }

    function handleReset() {
        setFilters(defaultFilters);
        write(defaultFilters);
        load(page => getGovTenders({ page, per_page: LIMIT }));
    }

    const inputClass = "px-2 py-1.5 text-sm rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

    return (
        <div className="flex min-h-0 flex-1 flex-col">
            <div className="flex min-h-0 flex-1 flex-col gap-4">
                <PageHeader title="政府標案" description="由排程每日抓取政府電子採購網，前台唯讀" />

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
                            className="px-4 py-1.5 text-sm font-medium rounded-sm bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors"
                        >
                            搜尋
                        </button>
                        <button
                            onClick={handleReset}
                            disabled={isPending}
                            className="px-4 py-1.5 text-sm font-medium rounded-sm bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                        >
                            重設
                        </button>
                    </div>
                </div>

                <ErrorBanner message={failed ? LOAD_FAILED : null} />

                <div className={`flex min-h-0 flex-1 flex-col transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                    <AdminTableContainer stickyHead fill>
                        {/* table-fixed：原本只有標案名稱有 min-w / max-w，其餘四欄任 auto layout
                            按 min-content 分配。視窗一窄（側欄收起、瀏覽器放大）表格就被撐過容器寬度，
                            把關鍵字欄推出右邊。固定配寬讓標案名稱吃剩餘空間，其餘欄不再互搶。 */}
                        <AdminTable className="text-sm table-fixed">
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh className="col-date">公告日</AdminTh>
                                    <AdminTh className="w-[9em] hidden sm:table-cell">類型</AdminTh>
                                    <AdminTh>標案名稱</AdminTh>
                                    <AdminTh className="w-[9em]">機關</AdminTh>
                                    <AdminTh className="w-[15em] hidden lg:table-cell">廠商</AdminTh>
                                    <AdminTh className="w-[6em]">關鍵字</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {tenders.length === 0 ? (
                                    <AdminEmptyRow colSpan={6}>
                                        {isPending ? '載入中…' : '目前沒有標案資料（排程每日抓取一次）'}
                                    </AdminEmptyRow>
                                ) : (
                                    tenders.map(t => (
                                        <AdminRow key={t.id}>
                                            <AdminTd className="whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400">
                                                {t.date}
                                            </AdminTd>
                                            <AdminTd className="text-xs hidden sm:table-cell">
                                                {t.tender_type}
                                            </AdminTd>
                                            <AdminTd className="wrap-break-word">
                                                {/* 標題會折行，所以連結**不能**用 inline-flex —— 那種盒子只以第一行
                                                    參與行框的基線計算，其餘行會溢出行框、疊到下面的 category 上。
                                                    純 inline 流 + inline-block 圖示，圖示自然跟在最後一行後面。 */}
                                                <a
                                                    href={t.detail_url}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    className="text-primary-700 dark:text-primary-300 hover:underline"
                                                >
                                                    {t.title}
                                                    <ExternalLink className="ml-1 inline-block h-3.5 w-3.5 align-[-2px]" />
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
                                            <AdminTd className="text-xs text-neutral-500 dark:text-neutral-400">
                                                {t.keyword}
                                            </AdminTd>
                                        </AdminRow>
                                    ))
                                )}
                            </tbody>
                        </AdminTable>
                    </AdminTableContainer>
                </div>

                {hasMore && (
                    <div className="flex shrink-0 justify-center">
                        <button
                            onClick={loadMore}
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
