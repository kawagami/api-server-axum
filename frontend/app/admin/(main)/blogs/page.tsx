import { Suspense } from 'react';
import type { Metadata } from 'next';
import { getBlogTagCounts } from '@/api/blogs';
import BlogsClient from './blogs-client';
import { SKELETON_HEADERS } from './skeleton-headers';
import { ListTableSkeleton } from '@/components/loading/table-skeleton';
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "文章",
    description: "部落格文章與標籤管理",
};

export default async function BlogsPage() {
    await requirePermission("blog:read");
    const permissions = await getMyPermissions();
    // 建立/存檔與 tag 改名都需要 blog:update；刪除另有 blog:delete（沒權限就不要給按了才 403 的按鈕）
    const canManageTags = permissions.includes("blog:update");
    // tag 篇數為全站統計；只有具改名/刪除權限時才載入與顯示管理面板（篩選下拉沿用同一份）
    const tags = canManageTags ? await getBlogTagCounts() : [];

    // 外層 padding 與寬度由 admin layout 給，這裡不要再包一層 padding / 底色。
    // BlogsClient 讀 useSearchParams（?q=&tag=&sort=）當初始條件，需要 Suspense 邊界
    return (
        <Suspense fallback={<ListTableSkeleton headers={SKELETON_HEADERS} rows={10} />}>
            <BlogsClient
                tags={tags}
                canCreate={canManageTags}
                canDelete={permissions.includes("blog:delete")}
                canManageTags={canManageTags}
            />
        </Suspense>
    );
}
