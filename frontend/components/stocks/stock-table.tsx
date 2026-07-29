import StockTableRow from "@/components/stocks/stock-table-row";
import { AdminTable, AdminHeadRow, AdminTh, AdminEmptyRow } from "@/components/admin/table";
import type { StockChange } from "@/types";

export default function StockTable({ data }: { data: StockChange[] }) {
    return (
        <AdminTable className="text-sm">
            <thead>
                <AdminHeadRow>
                    <AdminTh>股票代號</AdminTh>
                    <AdminTh>股票名稱</AdminTh>
                    <AdminTh>狀態</AdminTh>
                    <AdminTh>起始日期</AdminTh>
                    <AdminTh className="text-right">起始價格</AdminTh>
                    <AdminTh>結束日期</AdminTh>
                    <AdminTh className="text-right">結束價格</AdminTh>
                    <AdminTh className="text-right">變動 (%)</AdminTh>
                    <AdminTh>操作</AdminTh>
                </AdminHeadRow>
            </thead>
            <tbody>
                {data.length === 0 ? (
                    <AdminEmptyRow colSpan={9}>沒有符合條件的異動資料</AdminEmptyRow>
                ) : (
                    data.map((stock) => (
                        <StockTableRow key={`${stock.stock_no}${stock.start_date}${stock.end_date}`} stock={stock} />
                    ))
                )}
            </tbody>
        </AdminTable>
    );
}
