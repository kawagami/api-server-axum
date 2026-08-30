import { Suspense } from "react";
import { Loader2, ChevronLeft, ChevronRight } from "lucide-react";
import Link from "next/link";
import type { Metadata } from "next";
import StockTable from "@/components/stocks/stock-table";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { StatusLink } from "./status-link";
import { getStockChanges } from "@/app/admin/(main)/stocks/actions";
import type { StockChange } from "@/types";

export const metadata: Metadata = {
    title: "異動列表",
    description: "股票異動查詢結果",
};

const PER_PAGE = 50;

const STATUS_TABS: { value: string; label: string }[] = [
    { value: "", label: "全部" },
    { value: "completed", label: "已完成" },
    { value: "failed", label: "失敗" },
    { value: "pending", label: "等待中" },
];

function buildHref(status: string | undefined, page: number) {
    const params = new URLSearchParams();
    if (status) params.append("status", status);
    if (page > 1) params.append("page", String(page));
    const qs = params.toString();
    return `/admin/stocks/list${qs ? `?${qs}` : ""}`;
}

async function StockContent({ status, page }: { status: string | undefined; page: number }) {
    const { data, total } = await getStockChanges(status ?? null, page, PER_PAGE);
    const totalChange = data.reduce((sum: number, item: StockChange) => sum + (item.change ?? 0), 0);
    const totalCount = data.reduce((sum: number, item: StockChange) => sum + (item.change ? 1 : 0), 0);
    const offset = (page - 1) * PER_PAGE;
    const hasPrev = page > 1;
    const hasNext = offset + PER_PAGE < total;

    return (
        <>
            {/* 這些是統計值不是標題，用 dl 而不是一排 h1 */}
            <dl className="flex flex-wrap gap-x-6 gap-y-1 text-sm text-neutral-600 dark:text-neutral-400">
                <div className="flex gap-1">
                    <dt>共</dt>
                    <dd className="font-semibold text-neutral-800 dark:text-neutral-100">{total} 筆</dd>
                    <dt>，本頁</dt>
                    <dd className="font-semibold text-neutral-800 dark:text-neutral-100">{data.length} 筆</dd>
                </div>
                <div className="flex gap-1">
                    <dt>總變動</dt>
                    <dd className="font-semibold text-neutral-800 dark:text-neutral-100">{totalChange.toFixed(2)} %</dd>
                </div>
                <div className="flex gap-1">
                    <dt>有資料</dt>
                    <dd className="font-semibold text-neutral-800 dark:text-neutral-100">{totalCount} 筆</dd>
                </div>
                <div className="flex gap-1">
                    <dt>平均</dt>
                    <dd className="font-semibold text-neutral-800 dark:text-neutral-100">
                        {totalCount > 0 ? (totalChange / totalCount).toFixed(2) : '—'} %
                    </dd>
                </div>
            </dl>
            <AdminTableContainer stickyHead fill>
                <StockTable data={data} />
            </AdminTableContainer>
            <div className="flex items-center justify-between">
                <span className="text-sm text-neutral-500 dark:text-neutral-400">
                    {offset + 1}–{Math.min(offset + PER_PAGE, total)} / {total}
                </span>
                <div className="flex gap-2">
                    {hasPrev ? (
                        <Link
                            href={buildHref(status, page - 1)}
                            className="flex items-center gap-1 px-3 py-1.5 rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm transition-colors"
                        >
                            <ChevronLeft className="w-4 h-4" /> 上一頁
                        </Link>
                    ) : (
                        <span className="flex items-center gap-1 px-3 py-1.5 rounded-sm border border-neutral-200 dark:border-neutral-700 text-neutral-300 dark:text-neutral-600 text-sm cursor-not-allowed">
                            <ChevronLeft className="w-4 h-4" /> 上一頁
                        </span>
                    )}
                    {hasNext ? (
                        <Link
                            href={buildHref(status, page + 1)}
                            className="flex items-center gap-1 px-3 py-1.5 rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-900 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm transition-colors"
                        >
                            下一頁 <ChevronRight className="w-4 h-4" />
                        </Link>
                    ) : (
                        <span className="flex items-center gap-1 px-3 py-1.5 rounded-sm border border-neutral-200 dark:border-neutral-700 text-neutral-300 dark:text-neutral-600 text-sm cursor-not-allowed">
                            下一頁 <ChevronRight className="w-4 h-4" />
                        </span>
                    )}
                </div>
            </div>
        </>
    );
}

function StockContentSkeleton() {
    return (
        <div className="flex items-center justify-center py-16">
            <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
        </div>
    );
}

export default async function List({ searchParams }: { searchParams: Promise<{ status?: string; page?: string }> }) {
    const { status, page: pageStr } = await searchParams;
    const page = Math.max(1, Number(pageStr ?? 1) || 1);

    return (
        <div className="w-full flex min-h-0 flex-1 flex-col gap-4">
            <PageHeader title="股票異動列表" />
            <div className="flex gap-2 overflow-x-auto">
                {STATUS_TABS.map(({ value, label }) => (
                    <StatusLink key={value || 'all'} status={value} currentStatus={status ?? ''}>
                        {label}
                    </StatusLink>
                ))}
            </div>
            <Suspense key={`${status ?? ''}-${page}`} fallback={<StockContentSkeleton />}>
                <StockContent status={status} page={page} />
            </Suspense>
        </div>
    );
}
