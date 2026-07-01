"use client";

import { fetchStockClosingPricePair } from "@/app/admin/(main)/stocks/actions";
import { useActionState } from "react";
import { Loader2 } from "lucide-react";

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
            <div className="bg-primary-50 dark:bg-primary-900/30 p-3 rounded mb-4 dark:text-neutral-200">
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
        <div className="w-full p-3 sm:p-6 bg-neutral-100 dark:bg-neutral-800">
            <form action={formAction} className="space-y-4 bg-white dark:bg-neutral-900 p-4 rounded shadow">
                <div className="flex flex-col space-y-2">
                    <label htmlFor="stockNo" className="font-medium dark:text-neutral-200">股票代號</label>
                    <input type="text" name="stockNo" id="stockNo" className="border dark:border-neutral-600 p-2 rounded w-full bg-white dark:bg-neutral-700 dark:text-neutral-200" placeholder="例如: 3036" />
                </div>
                <div className="flex flex-col space-y-2">
                    <label htmlFor="start_date" className="font-medium dark:text-neutral-200">起始日期 (YYYYMMDD 或 民國)</label>
                    <input type="text" name="start_date" id="start_date" className="border dark:border-neutral-600 p-2 rounded w-full bg-white dark:bg-neutral-700 dark:text-neutral-200" placeholder="1130101 或 20240101" />
                </div>
                <div className="flex flex-col space-y-2">
                    <label htmlFor="end_date" className="font-medium dark:text-neutral-200">結束日期 (YYYYMMDD 或 民國)</label>
                    <input type="text" name="end_date" id="end_date" className="border dark:border-neutral-600 p-2 rounded w-full bg-white dark:bg-neutral-700 dark:text-neutral-200" placeholder="1130430 或 20240430" defaultValue={new Date().toISOString().slice(0, 10).replace(/-/g, "")} />
                </div>
                <button type="submit" disabled={isPending} className="flex items-center gap-1 bg-primary-600 text-white px-4 py-2 rounded hover:bg-primary-700 disabled:opacity-50">
                    {isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                    {isPending ? "查詢中…" : "查詢"}
                </button>
            </form>
            {state.error && <div className="bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-400 p-3 rounded">{state.error}</div>}
            {state.stockData?.prices && (
                <div className="bg-white dark:bg-neutral-900 p-4 rounded shadow">
                    <h2 className="text-lg font-semibold mb-3 dark:text-white">查詢結果</h2>
                    {renderStats(state.stockData)}
                    <div className="overflow-x-auto">
                    <table className="w-full table-auto border-collapse">
                        <thead>
                            <tr>
                                <th className="border dark:border-neutral-600 px-2 py-1 dark:text-neutral-200">股票代號</th>
                                <th className="border dark:border-neutral-600 px-2 py-1 dark:text-neutral-200">日期</th>
                                <th className="border dark:border-neutral-600 px-2 py-1 dark:text-neutral-200">收盤價</th>
                            </tr>
                        </thead>
                        <tbody>
                            {state.stockData.prices.map((item) => (
                                <tr key={item.date} className="dark:text-neutral-200">
                                    <td className="border dark:border-neutral-600 px-2 py-1 text-center">{item.stock_no}</td>
                                    <td className="border dark:border-neutral-600 px-2 py-1 text-center">{item.date}</td>
                                    <td className="border dark:border-neutral-600 px-2 py-1 text-center">{item.close_price}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    </div>
                </div>
            )}
        </div>
    );
}
