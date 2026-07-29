import { Suspense } from "react";
import VocabAdminClient from "./vocab-admin-client";
import type { Metadata } from "next";
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";
import { ListTableSkeleton } from "@/components/loading/table-skeleton";

const SKELETON_HEADERS = ['語言', '表記', '讀音', '釋義', '詞性', '難度', '✗/✓', '狀態', '操作'];

export const metadata: Metadata = {
    title: "單字題庫",
    description: "單字闖關題庫管理",
};

export default async function VocabAdminPage() {
    await requirePermission("vocab:read");
    const canUpdate = (await getMyPermissions()).includes("vocab:update");
    // VocabAdminClient 讀 useSearchParams（?language=&difficulty=…）當初始條件，需要 Suspense 邊界
    return (
        <Suspense fallback={<ListTableSkeleton headers={SKELETON_HEADERS} rows={10} />}>
            <VocabAdminClient canUpdate={canUpdate} />
        </Suspense>
    );
}
