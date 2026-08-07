import type { Metadata } from "next";
import { getStockDayAll } from "@/app/admin/(main)/stocks/actions";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "當日全部",
    description: "全市場每日行情查詢",
};

const inputClass = "px-2 py-1.5 text-sm rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100";

export default async function Page({ searchParams }: { searchParams: Promise<{ trade_date?: string; stock_code?: string; page?: string; per_page?: string }> }) {
    const params = await searchParams;
    const trade_date = params.trade_date ?? "";
    const stock_code = params.stock_code ?? "";
    const page = Math.max(1, parseInt(params.page ?? "1", 10) || 1);
    const perPage = parseInt(params.per_page ?? "100", 10);

    const { data, total } = await getStockDayAll({ trade_date, stock_code, page, perPage });

    return (
        <div className="w-full flex flex-col gap-4">
            <PageHeader title="全市場行情" description={`共 ${total} 筆，本頁 ${data.length} 筆`} />

            <form method="get" className="flex flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">交易日期</label>
                    <input type="text" name="trade_date" defaultValue={trade_date} placeholder="YYYYMMDD" className={inputClass} />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">股票代號</label>
                    <input type="text" name="stock_code" defaultValue={stock_code} placeholder="2330" className={inputClass} />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">筆數</label>
                    <input type="number" name="per_page" defaultValue={perPage} min={1} max={200} className={`${inputClass} w-20`} />
                </div>
                <div className="flex flex-col gap-1">
                    <label className="text-xs text-neutral-500 dark:text-neutral-400">頁碼</label>
                    <input type="number" name="page" defaultValue={page} min={1} className={`${inputClass} w-20`} />
                </div>
                <button type="submit" className="px-4 py-1.5 text-sm font-medium rounded-sm bg-primary-600 hover:bg-primary-700 text-white transition-colors">
                    查詢
                </button>
            </form>

            <AdminTableContainer stickyHead>
                <AdminTable className="text-sm">
                    <thead>
                        <AdminHeadRow>
                            <AdminTh>交易日期</AdminTh>
                            <AdminTh>股票代號</AdminTh>
                            <AdminTh>股票名稱</AdminTh>
                            <AdminTh className="text-right">開盤</AdminTh>
                            <AdminTh className="text-right">最高</AdminTh>
                            <AdminTh className="text-right">最低</AdminTh>
                            <AdminTh className="text-right">收盤</AdminTh>
                            <AdminTh className="text-right">漲跌</AdminTh>
                            <AdminTh className="text-right">成交量</AdminTh>
                            <AdminTh className="text-right hidden sm:table-cell">成交金額</AdminTh>
                            <AdminTh className="text-right hidden sm:table-cell">成交筆數</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {data.length === 0 ? (
                            <AdminEmptyRow colSpan={11}>沒有符合條件的行情資料</AdminEmptyRow>
                        ) : (
                            data.map((item, i) => (
                                <AdminRow key={`${item.stock_code}-${i}`}>
                                    <AdminTd>{item.trade_date}</AdminTd>
                                    <AdminTd>{item.stock_code}</AdminTd>
                                    <AdminTd>{item.stock_name}</AdminTd>
                                    <AdminTd className="text-right">{item.open_price}</AdminTd>
                                    <AdminTd className="text-right">{item.high_price}</AdminTd>
                                    <AdminTd className="text-right">{item.low_price}</AdminTd>
                                    <AdminTd className="text-right">{item.close_price}</AdminTd>
                                    <AdminTd className="text-right">{item.price_change}</AdminTd>
                                    <AdminTd className="text-right">{item.trade_volume?.toLocaleString()}</AdminTd>
                                    <AdminTd className="text-right hidden sm:table-cell">{item.trade_amount?.toLocaleString()}</AdminTd>
                                    <AdminTd className="text-right hidden sm:table-cell">{item.transaction_count?.toLocaleString()}</AdminTd>
                                </AdminRow>
                            ))
                        )}
                    </tbody>
                </AdminTable>
            </AdminTableContainer>

            <div className="flex gap-2">
                {page > 1 && (
                    <a
                        href={`?trade_date=${trade_date}&stock_code=${stock_code}&per_page=${perPage}&page=${page - 1}`}
                        className="px-4 py-2 rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm transition-colors"
                    >
                        上一頁
                    </a>
                )}
                {/* 有 total 就能精確判斷還有沒有下一頁；靠「本頁剛好滿」猜的話，
                    最後一頁滿版時會多出一顆按了是空白的「下一頁」 */}
                {page * perPage < total && (
                    <a
                        href={`?trade_date=${trade_date}&stock_code=${stock_code}&per_page=${perPage}&page=${page + 1}`}
                        className="px-4 py-2 rounded-sm border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-700 text-sm transition-colors"
                    >
                        下一頁
                    </a>
                )}
            </div>
        </div>
    );
}
