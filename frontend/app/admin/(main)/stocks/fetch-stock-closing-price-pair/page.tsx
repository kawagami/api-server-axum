"use client";

import { fetchStockClosingPricePair } from "@/app/admin/(main)/stocks/actions";
import { useActionState } from "react";
import { Loader2 } from "lucide-react";
import PageHeader from "@/components/admin/page-header";
import ErrorBanner from "@/components/admin/error-banner";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";

const inputClass = "w-full px-3 py-2 text-sm rounded-lg border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

interface StockPriceItem {
    stock_no: string;
    date: string;
    close_price: number;
}

interface StockStats {
    price_diff: number;
    percent_change: number;
    is_increase: boolean;
    day_span: number;
}

interface StockPriceData {
    prices: StockPriceItem[];
    stats: StockStats;
}

interface FetchState {
    stockData: StockPriceData | null;
    error: string | null;
}

function parseDateInput(input: string): string | null {
    const trimmed = input.trim();
    if (/^\d{7}$/.test(trimmed)) {
        const year = parseInt(trimmed.slice(0, 3), 10);
        return `${year + 1911}${trimmed.slice(3)}`;
    }
    if (/^\d{8}$/.test(trimmed)) return trimmed;
    return null;
}

function yyyymmddToDate(yyyymmdd: string): Date {
    return new Date(parseInt(yyyymmdd.slice(0, 4)), parseInt(yyyymmdd.slice(4, 6)) - 1, parseInt(yyyymmdd.slice(6, 8)));
}

async function fetchStockData(prevState: FetchState, formData: FormData): Promise<FetchState> {
    const stockNo = formData.get("stockNo")?.toString().trim();
    const startInput = formData.get("start_date")?.toString().trim();
    const endInput = formData.get("end_date")?.toString().trim();

    if (!stockNo || !startInput || !endInput) return { stockData: null, error: "請輸入完整資料！" };

    const start_date = parseDateInput(startInput);
    const end_date = parseDateInput(endInput);

    if (!start_date || !end_date) return { stockData: null, error: "日期格式錯誤，請輸入 7 碼民國年或 8 碼西元年" };

    const today = new Date();
    today.setHours(0, 0, 0, 0);
    if (yyyymmddToDate(start_date) > today || yyyymmddToDate(end_date) > today) {
        return { stockData: null, error: "日期不能超過今天，請重新輸入。" };
    }

    try {
        const data = await fetchStockClosingPricePair({ stock_no: stockNo, start_date, end_date });
        return { stockData: data as StockPriceData, error: null };
    } catch (error) {
        return { stockData: null, error: (error as Error).message || "無法取得資料，請稍後再試。" };
    }
}

export default function Search() {
    const [state, formAction, isPending] = useActionState(fetchStockData, { stockData: null, error: null });

    const renderStats = (data: StockPriceData) => {
        if (!data?.stats) return null;
        const { price_diff, percent_change, is_increase, day_span } = data.stats;
        const priceChangeClass = is_increase ? "text-green-600" : "text-red-600";
        return (
            <div className="bg-primary-50 dark:bg-primary-900/30 p-3 rounded-sm text-sm text-neutral-800 dark:text-neutral-200">
                {data.prices.length >= 2 && (
                    <>
                        <p><span className="font-medium">起始收盤價：</span> {data.prices[0].close_price}</p>
                        <p><span className="font-medium">結束收盤價：</span> {data.prices[data.prices.length - 1].close_price}</p>
                    </>
                )}
                <p><span className="font-medium">漲跌點數：</span><span className={priceChangeClass}>{price_diff.toFixed(2)}</span></p>
                <p><span className="font-medium">漲跌幅 (%)：</span><span className={priceChangeClass}>{percent_change.toFixed(2)}%</span></p>
                <p><span className="font-medium">經過天數：</span> {day_span} 天</p>
            </div>
        );
    };

    return (
        <div className="w-full max-w-2xl flex flex-col gap-4">
            <PageHeader title="收盤價查詢" description="查詢特定股票在指定區間的收盤價與漲跌" />

            <form action={formAction} className="space-y-4 bg-white dark:bg-neutral-900 p-4 sm:p-6 rounded-lg shadow-lg">
                <div className="flex flex-col gap-1">
                    <label htmlFor="stockNo" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">股票代號</label>
                    <input type="text" name="stockNo" id="stockNo" className={inputClass} placeholder="例如：3036" />
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="start_date" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">起始日期（YYYYMMDD 或民國年）</label>
                    <input type="text" name="start_date" id="start_date" className={inputClass} placeholder="1130101 或 20240101" />
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="end_date" className="text-sm font-medium text-neutral-700 dark:text-neutral-300">結束日期（YYYYMMDD 或民國年）</label>
                    <input type="text" name="end_date" id="end_date" className={inputClass} placeholder="1130430 或 20240430" defaultValue={new Date().toISOString().slice(0, 10).replace(/-/g, "")} />
                </div>
                <button type="submit" disabled={isPending} className="flex items-center gap-1 bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700 disabled:opacity-50 text-sm font-medium transition-colors">
                    {isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                    {isPending ? "查詢中…" : "查詢"}
                </button>
            </form>

            <ErrorBanner message={state.error} />

            {state.stockData?.prices && (
                <div className="bg-white dark:bg-neutral-900 p-4 sm:p-6 rounded-lg shadow-lg flex flex-col gap-3">
                    <h2 className="text-lg font-semibold text-neutral-800 dark:text-neutral-100">查詢結果</h2>
                    {renderStats(state.stockData)}
                    <div className="overflow-x-auto">
                        <AdminTable className="text-sm">
                            <thead>
                                <AdminHeadRow>
                                    <AdminTh>股票代號</AdminTh>
                                    <AdminTh>日期</AdminTh>
                                    <AdminTh className="text-right">收盤價</AdminTh>
                                </AdminHeadRow>
                            </thead>
                            <tbody>
                                {state.stockData.prices.length === 0 ? (
                                    <AdminEmptyRow colSpan={3}>這段區間沒有資料</AdminEmptyRow>
                                ) : (
                                    state.stockData.prices.map((item) => (
                                        <AdminRow key={item.date}>
                                            <AdminTd>{item.stock_no}</AdminTd>
                                            <AdminTd>{item.date}</AdminTd>
                                            <AdminTd className="text-right">{item.close_price}</AdminTd>
                                        </AdminRow>
                                    ))
                                )}
                            </tbody>
                        </AdminTable>
                    </div>
                </div>
            )}
        </div>
    );
}
