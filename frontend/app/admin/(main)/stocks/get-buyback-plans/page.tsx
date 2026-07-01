import { getStockBuybackPeriods } from "@/app/admin/(main)/stocks/actions";

export default async function Page() {
    const data = await getStockBuybackPeriods();

    return (
        <div className="w-full p-3 sm:p-6 bg-neutral-100 dark:bg-neutral-800">
            <h1 className="text-xl font-bold mb-4 dark:text-white">庫藏股計畫清單</h1>
            <div className="text-sm text-neutral-600 dark:text-neutral-400 mb-2">共 {data.length} 筆</div>
            <div className="overflow-x-auto">
                <table className="min-w-full bg-white dark:bg-neutral-900 border border-neutral-300 dark:border-neutral-600 text-sm">
                    <thead className="bg-neutral-200 dark:bg-neutral-700 sticky top-0 z-10">
                        <tr>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">股票代號</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">起始日</th>
                            <th className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 dark:text-neutral-200">結束日</th>
                        </tr>
                    </thead>
                    <tbody>
                        {data.map((item, i) => (
                            <tr key={i} className="hover:bg-neutral-50 dark:hover:bg-neutral-800 dark:text-neutral-200">
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.stock_no}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.start_date}</td>
                                <td className="px-4 py-2 border border-neutral-300 dark:border-neutral-600 text-center">{item.end_date}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
