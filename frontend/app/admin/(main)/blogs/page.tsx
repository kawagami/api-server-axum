import type { Metadata } from 'next';
import { getAdminBlogs, getBlogTagCounts } from '@/api/blogs';
import { CreateButton, EditButton, DeleteButton } from '@/components/blogs/blog-action-buttons';
import TagManager from '@/components/blogs/tag-manager';
import PageHeader from '@/components/admin/page-header';
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";

export const metadata: Metadata = {
    title: "文章",
    description: "部落格文章與標籤管理",
};

export default async function BlogsPage() {
    await requirePermission("blog:read");
    const [{ data: blogs }, permissions] = await Promise.all([
        getAdminBlogs({ per_page: 200 }),
        getMyPermissions(),
    ]);
    const canManageTags = permissions.includes("blog:update");
    // tag 篇數為全站統計；只有具改名/刪除權限時才載入與顯示管理面板
    const tags = canManageTags ? await getBlogTagCounts() : [];

    // 外層 padding 與寬度由 admin layout 給，這裡不要再包一層 padding / 底色
    return (
        <div className="w-full flex flex-col gap-4">
            <PageHeader
                title="文章"
                description={`共 ${blogs.length} 篇`}
                actions={<CreateButton />}
            />
            <div className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg p-4 sm:p-6">
                {blogs.length > 0 ? (
                    <ul className="divide-y divide-neutral-200 dark:divide-neutral-700">
                        {blogs.map((blog) => (
                            <li key={blog.id} className="flex items-center justify-between p-4 hover:bg-neutral-50 dark:hover:bg-neutral-800 transition-colors">
                                <span className="text-neutral-800 dark:text-neutral-100 font-medium min-w-0 flex-1 truncate">
                                    {blog.tocs[0] || '未命名文章'}
                                </span>
                                <div className="flex space-x-2 shrink-0 ml-2">
                                    <EditButton uuid={blog.id} />
                                    <DeleteButton uuid={blog.id} />
                                </div>
                            </li>
                        ))}
                    </ul>
                ) : (
                    <p className="text-neutral-500 dark:text-neutral-400 text-center py-8">目前沒有文章</p>
                )}
            </div>
            {canManageTags && <TagManager tags={tags} />}
        </div>
    );
}
