import { getStockDayAll } from "@/app/admin/(main)/stocks/actions";

export default async function Page({ searchParams }: { searchParams: Promise<{ trade_date?: string; stock_code?: string; page?: string; per_page?: string }> }) {
    const params = await searchParams;
    const trade_date = params.trade_date ?? "";
    const stock_code = params.stock_code ?? "";
    const page = Math.max(1, parseInt(params.page ?? "1", 10) || 1);
    const perPage = parseInt(params.per_page ?? "100", 10);

    const data = await getStockDayAll({ trade_date, stock_code, page, perPage });

    return (
        <div className="w-full p-3 sm:p-6 bg-neutral-100 dark:bg-neutral-800">
            <h1 className="text-xl font-bold mb-4 dark:text-white">全市場行情</h1>
            <form method="get" className="flex gap-2 mb-4 items-end flex-wrap">
                <div className="flex flex-col gap-1">
                    <label className="text-sm text-neutral-600 dark:text-neutral-400">交易日期</label>
                    <input type="text" name="trade_date" defaultValue={trade_date} placeholder="YYYYMMDD" className="border dark:border-neutral-600 px-2 py-1 text-sm rounded bg-white dark:bg-neutral-700 dark:text-neutral-200" />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm text-neutral-600 dark:text-neutral-400">股票代號</label>
                    <input type="text" name="stock_code" defaultValue={stock_code} placeholder="2330" className="border dark:border-neutral-600 px-2 py-1 text-sm rounded bg-white dark:bg-neutral-700 dark:text-neutral-200" />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm text-neutral-600 dark:text-neutral-400">筆數</label>
                    <input type="number" name="per_page" defaultValue={perPage} min={1} max={200} className="border dark:border-neutral-600 px-2 py-1 text-sm rounded w-20 bg-white dark:bg-neutral-700 dark:text-neutral-200" />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-sm text-neutral-600 dark:text-neutral-400">頁碼</label>
                    <input type="number" name="page" defaultValue={page} min={1} className="border dark:border-neutral-600 px-2 py-1 text-sm rounded w-20 bg-white dark:bg-neutral-700 dark:text-neutral-200" />
                </div>
                <button type="submit" className="px-3 py-1 bg-primary-600 text-white text-sm rounded hover:bg-primary-700">查詢</button>
            </form>
            <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-2">共 {data.length} 筆</div>
            <div className="overflow-x-auto">
                <table className="min-w-full bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-600 text-sm">
                    <thead className="bg-neutral-200 dark:bg-neutral-700 sticky top-0 z-10">
                        <tr>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">交易日期</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">股票代號</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">股票名稱</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">開盤</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">最高</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">最低</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">收盤</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">漲跌</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">成交量</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">成交金額</th>
                            <th className="px-3 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">成交筆數</th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.map((item, i) => (
                            <tr key={`${item.stock_code}-${i}`} className="hover:bg-neutral-50 dark:hover:bg-neutral-700 dark:text-neutral-200">
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-center">{item.trade_date}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-center">{item.stock_code}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600">{item.stock_name}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.open_price}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.high_price}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.low_price}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.close_price}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.price_change}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.trade_volume?.toLocaleString()}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.trade_amount?.toLocaleString()}</td>
                                <td className="px-3 py-1 border border-neutral-300 dark:border-neutral-600 text-right">{item.transaction_count?.toLocaleString()}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
            <div className="flex gap-2 mt-4">
                {page > 1 && (
                    <a href={`?trade_date=${trade_date}&stock_code=${stock_code}&per_page=${perPage}&page=${page - 1}`} className="px-3 py-1 bg-neutral-300 dark:bg-neutral-600 dark:text-neutral-200 rounded text-sm hover:bg-neutral-400 dark:hover:bg-neutral-500">上一頁</a>
                )}
                {data.length === perPage && (
                    <a href={`?trade_date=${trade_date}&stock_code=${stock_code}&per_page=${perPage}&page=${page + 1}`} className="px-3 py-1 bg-neutral-300 dark:bg-neutral-600 dark:text-neutral-200 rounded text-sm hover:bg-neutral-400 dark:hover:bg-neutral-500">下一頁</a>
                )}
            </div>
        </div>
    );
}
