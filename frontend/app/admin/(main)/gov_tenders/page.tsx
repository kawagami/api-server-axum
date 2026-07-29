import { Suspense } from "react";
import GovTendersClient from "./gov-tenders-client";
import type { Metadata } from "next";
import { requirePermission } from "@/libs/admin-permissions";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";

const SKELETON_HEADERS = ['公告日', '類型', '標案名稱', '機關', '廠商', '關鍵字'];

export const metadata: Metadata = {
    title: "政府標案",
    description: "政府電子採購網標案追蹤",
};

export default async function GovTendersPage() {
    await requirePermission("gov_tender:read");
    // GovTendersClient 讀 useSearchParams（?q=&keyword=&tender_type=）當初始條件，需要 Suspense 邊界
    return (
        <Suspense fallback={<ListTableSkeleton headers={SKELETON_HEADERS} rows={10} />}>
            <GovTendersClient />
        </Suspense>
    );
}
