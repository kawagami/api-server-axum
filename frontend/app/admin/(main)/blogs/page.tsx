import { getAdminBlogs, getBlogTagCounts } from '@/api/blogs';
import { CreateButton, EditButton, DeleteButton } from '@/components/blogs/blog-action-buttons';
import TagManager from '@/components/blogs/tag-manager';
import { requirePermission, getMyPermissions } from "@/libs/admin-permissions";

export default async function BlogsPage() {
    await requirePermission("blog:read");
    const [{ data: blogs }, permissions] = await Promise.all([
        getAdminBlogs({ per_page: 200 }),
        getMyPermissions(),
    ]);
    const canManageTags = permissions.includes("blog:update");
    // tag 篇數為全站統計；只有具改名/刪除權限時才載入與顯示管理面板
    const tags = canManageTags ? await getBlogTagCounts() : [];

    return (
        <div className="w-full p-3 sm:p-6 bg-neutral-100 dark:bg-neutral-900">
            <div className="mb-8 flex justify-center">
                <CreateButton />
            </div>
            <div className="bg-white dark:bg-neutral-800 shadow rounded-lg p-6">
                {blogs.length > 0 ? (
                    <ul className="divide-y divide-neutral-200 dark:divide-neutral-700">
                        {blogs.map((blog) => (
                            <li key={blog.id} className="flex items-center justify-between p-4 hover:bg-neutral-50 dark:hover:bg-neutral-700 transition">
                                <span className="text-neutral-800 dark:text-neutral-100 font-medium min-w-0 flex-1 truncate">
                                    {blog.tocs[0] || '未命名 blog'}
                                </span>
                                <div className="flex space-x-2 shrink-0 ml-2">
                                    <EditButton uuid={blog.id} />
                                    <DeleteButton uuid={blog.id} />
                                </div>
                            </li>
                        ))}
                    </ul>
                ) : (
                    <p className="text-neutral-500 dark:text-neutral-400 text-center">暫無 blog 內容</p>
                )}
            </div>
            {canManageTags && <TagManager tags={tags} />}
        </div>
    );
}
