import { getUnfinishedBuybackPriceGap } from "@/app/admin/(main)/stocks/actions";

interface BuybackPriceGapItem {
    stock_no: string;
    stock_name: string;
    start_date: string;
    end_date: string;
    price_on_start_date: number;
    latest_price: number;
    diff: string;
    diff_percent: string;
}

export default async function Page() {
    const info = await getUnfinishedBuybackPriceGap() as BuybackPriceGapItem[];

    const totalDiffPercent = info.reduce((sum, item) => {
        const percent = parseFloat(item.diff_percent);
        return sum + (isNaN(percent) ? 0 : percent);
    }, 0);
    const avgDiffPercent = info.length > 0 ? totalDiffPercent / info.length : 0;

    return (
        <div className="w-full p-3 sm:p-6 bg-neutral-100 dark:bg-neutral-800">
            <h1 className="text-xl font-bold mb-4 dark:text-white">執行中的庫藏股</h1>
            <div className="mb-4 text-right text-sm text-neutral-600 dark:text-neutral-400 space-y-1">
                <div>資料數量：{info.length}</div>
                <div>價差 (%) 總和：{totalDiffPercent.toFixed(2)}%</div>
                <div>平均價差 (%)：{avgDiffPercent.toFixed(2)}%</div>
            </div>
            <div className="overflow-x-auto">
                <table className="min-w-full bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-600 text-sm">
                    <thead className="bg-neutral-200 dark:bg-neutral-700 sticky top-0 z-10">
                        <tr>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">股票代號</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">股票名稱</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">開始日</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">結束日</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">開始日價格</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">最新價格</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">價差</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">價差 (%)</th>
                        </tr>
                    </thead>
                    <tbody>
                        {info.map((item) => (
                            <tr key={item.stock_no} className="hover:bg-neutral-100 dark:hover:bg-neutral-700 dark:text-neutral-200">
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.stock_no}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.stock_name}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.start_date}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.end_date}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-right">{item.price_on_start_date}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-right">{item.latest_price}</td>
                                <td className={`px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-right ${parseFloat(item.diff) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>{item.diff}</td>
                                <td className={`px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-right ${parseFloat(item.diff_percent) >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}`}>{item.diff_percent}%</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
