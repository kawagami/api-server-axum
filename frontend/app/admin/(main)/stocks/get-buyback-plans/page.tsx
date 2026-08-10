import type { Metadata } from "next";
import { getStockBuybackPeriods } from "@/app/admin/(main)/stocks/actions";
import PageHeader from "@/components/admin/page-header";
import AdminTableContainer from "@/components/admin/admin-table-container";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";

export const metadata: Metadata = {
    title: "回購計畫",
    description: "庫藏股計畫清單",
};

export default async function Page() {
    const data = await getStockBuybackPeriods();

    return (
        <div className="w-full flex min-h-0 flex-1 flex-col gap-4">
            <PageHeader title="庫藏股計畫清單" description={`共 ${data.length} 筆`} />
            <AdminTableContainer stickyHead fill>
                <AdminTable className="text-sm">
                    <thead>
                        <AdminHeadRow>
                            <AdminTh>股票代號</AdminTh>
                            <AdminTh>起始日</AdminTh>
                            <AdminTh>結束日</AdminTh>
                        </AdminHeadRow>
                    </thead>
                    <tbody>
                        {data.length === 0 ? (
                            <AdminEmptyRow colSpan={3}>目前沒有庫藏股計畫</AdminEmptyRow>
                        ) : (
                            data.map((item, i) => (
                                <AdminRow key={i}>
                                    <AdminTd>{item.stock_no}</AdminTd>
                                    <AdminTd>{item.start_date}</AdminTd>
                                    <AdminTd>{item.end_date}</AdminTd>
                                </AdminRow>
                            ))
                        )}
                    </tbody>
                </AdminTable>
            </AdminTableContainer>
        </div>
    );
}
