import { patchStockPendingAction } from "@/app/admin/(main)/stocks/actions";
import { AdminRow, AdminTd } from "@/components/admin/table";
import { STOCK_STATUS_LABEL } from "@/libs/badge-styles";
import type { StockChange } from "@/types";

export default function StockTableRow({ stock }: { stock: StockChange }) {
    return (
        <AdminRow>
            <AdminTd>{stock.stock_no}</AdminTd>
            <AdminTd>{stock.stock_name}</AdminTd>
            <AdminTd>{STOCK_STATUS_LABEL[stock.status] ?? stock.status}</AdminTd>
            <AdminTd>{stock.start_date}</AdminTd>
            <AdminTd className="text-right">{stock.start_price}</AdminTd>
            <AdminTd>{stock.end_date}</AdminTd>
            <AdminTd className="text-right">{stock.end_price}</AdminTd>
            {/* 台股慣例：紅漲綠跌 */}
            <AdminTd className={`text-right ${stock.change < 0 ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}`}>
                {stock.change ? `${stock.change}%` : ``}
            </AdminTd>
            <AdminTd>
                <form action={patchStockPendingAction} className="inline">
                    <input type="hidden" name="id" value={String(stock.id)} />
                    <button
                        type="submit"
                        className="px-3 py-1.5 text-sm font-medium rounded-sm bg-primary-600 hover:bg-primary-700 text-white transition-colors"
                    >
                        再查詢
                    </button>
                </form>
            </AdminTd>
        </AdminRow>
    );
}
