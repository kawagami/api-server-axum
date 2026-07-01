import { Suspense } from "react";
import { Loader2, ChevronLeft, ChevronRight } from "lucide-react";
import Link from "next/link";
import StockTable from "@/components/stocks/stock-table";
import { StatusLink } from "./status-link";
import { getStockChanges } from "@/app/admin/(main)/stocks/actions";
import type { StockChange } from "@/types";

const PER_PAGE = 50;

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
            <div className="flex flex-wrap gap-x-6 gap-y-1 mb-4">
                <h1 className="text-xl font-bold dark:text-white">共 {total} 筆，本頁 {data.length} 筆</h1>
                <h1 className="text-xl font-bold dark:text-white">總變動 {totalChange.toFixed(2)} %</h1>
                <h1 className="text-xl font-bold dark:text-white">有資料 {totalCount} 筆</h1>
                <h1 className="text-xl font-bold dark:text-white">平均 {totalCount > 0 ? (totalChange / totalCount).toFixed(2) : '—'} %</h1>
            </div>
            <div className="overflow-x-auto">
                <StockTable data={data} />
            </div>
            <div className="flex items-center justify-between mt-4">
                <span className="text-sm text-neutral-500 dark:text-neutral-400">
                    {offset + 1}–{Math.min(offset + PER_PAGE, total)} / {total}
                </span>
                <div className="flex gap-2">
                    {hasPrev ? (
                        <Link
                            href={buildHref(status, page - 1)}
                            className="flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm"
                        >
                            <ChevronLeft className="w-4 h-4" /> 上一頁
                        </Link>
                    ) : (
                        <span className="flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-200 dark:border-neutral-700 text-neutral-300 dark:text-neutral-600 text-sm cursor-not-allowed">
                            <ChevronLeft className="w-4 h-4" /> 上一頁
                        </span>
                    )}
                    {hasNext ? (
                        <Link
                            href={buildHref(status, page + 1)}
                            className="flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm"
                        >
                            下一頁 <ChevronRight className="w-4 h-4" />
                        </Link>
                    ) : (
                        <span className="flex items-center gap-1 px-3 py-1.5 rounded border border-neutral-200 dark:border-neutral-700 text-neutral-300 dark:text-neutral-600 text-sm cursor-not-allowed">
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
        <div className="w-full p-3 sm:p-6 bg-neutral-100 dark:bg-neutral-800">
            <div className="flex gap-2 mb-4 overflow-x-auto">
                {["", "completed", "failed", "pending"].map((s) => (
                    <StatusLink key={s || 'all'} status={s} currentStatus={status ?? ''}>
                        {s || 'All'}
                    </StatusLink>
                ))}
            </div>
            <Suspense key={`${status ?? ''}-${page}`} fallback={<StockContentSkeleton />}>
                <StockContent status={status} page={page} />
            </Suspense>
        </div>
    );
}
