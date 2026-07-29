import type { Metadata } from "next";
import { getUnfinishedBuybackPriceGap } from "@/app/admin/(main)/stocks/actions";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "未完成回購",
    description: "執行中的庫藏股與價差",
};

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

// 台股慣例：紅漲綠跌（語意色，不走 primary/neutral token）
function changeClass(value: string) {
    return parseFloat(value) >= 0
        ? 'text-green-600 dark:text-green-400'
        : 'text-red-600 dark:text-red-400';
}

export default async function Page() {
    const info = await getUnfinishedBuybackPriceGap() as BuybackPriceGapItem[];

    const totalDiffPercent = info.reduce((sum, item) => {
        const percent = parseFloat(item.diff_percent);
        return sum + (isNaN(percent) ? 0 : percent);
    }, 0);
    const avgDiffPercent = info.length > 0 ? totalDiffPercent / info.length : 0;

    return (
        <div className="w-full flex flex-col gap-4">
            <PageHeader
                title="執行中的庫藏股"
                description={`共 ${info.length} 筆・價差總和 ${totalDiffPercent.toFixed(2)}%・平均 ${avgDiffPercent.toFixed(2)}%`}
            />
            <AdminTableContainer stickyHead>
                <AdminTable className="text-sm">
                    <thead>
                        <AdminHeadRow>
                            <AdminTh>股票代號</AdminTh>
                            <AdminTh>股票名稱</AdminTh>
                            <AdminTh>開始日</AdminTh>
                            <AdminTh>結束日</AdminTh>
                            <AdminTh className="text-right">開始日價格</AdminTh>
                            <AdminTh className="text-right">最新價格</AdminTh>
                            <AdminTh className="text-right">價差</AdminTh>
                            <AdminTh className="text-right">價差 (%)</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {info.length === 0 ? (
                            <AdminEmptyRow colSpan={8}>目前沒有執行中的庫藏股</AdminEmptyRow>
                        ) : (
                            info.map((item) => (
                                <AdminRow key={item.stock_no}>
                                    <AdminTd>{item.stock_no}</AdminTd>
                                    <AdminTd>{item.stock_name}</AdminTd>
                                    <AdminTd>{item.start_date}</AdminTd>
                                    <AdminTd>{item.end_date}</AdminTd>
                                    <AdminTd className="text-right">{item.price_on_start_date}</AdminTd>
                                    <AdminTd className="text-right">{item.latest_price}</AdminTd>
                                    <AdminTd className={`text-right ${changeClass(item.diff)}`}>{item.diff}</AdminTd>
                                    <AdminTd className={`text-right ${changeClass(item.diff_percent)}`}>{item.diff_percent}%</AdminTd>
                                </AdminRow>
                            ))
                        )}
                    </tbody>
                </AdminTable>
            </AdminTableContainer>
        </div>
    );
}
