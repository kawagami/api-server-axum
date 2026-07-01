import { getBlogs } from '@/api/blogs';
import { CreateButton, EditButton, DeleteButton } from '@/components/blogs/blog-action-buttons';

export default async function BlogsPage() {
    const { data: blogs } = await getBlogs({ per_page: 1000 });

    return (
        <div className="w-full p-6 bg-neutral-100 dark:bg-neutral-900">
            <div className="mb-8 flex justify-center">
                <CreateButton />
            </div>
            <div className="bg-white dark:bg-neutral-800 shadow rounded-lg p-6">
                {blogs.length > 0 ? (
                    <ul className="divide-y divide-neutral-200 dark:divide-neutral-700">
                        {blogs.map((blog) => (
                            <li key={blog.id} className="flex items-center justify-between p-4 hover:bg-neutral-50 dark:hover:bg-neutral-700 transition">
                                <span className="text-neutral-800 dark:text-neutral-100 font-medium truncate max-w-xs">
                                    {blog.tocs[0] || '未命名 blog'}
                                </span>
                                <div className="flex space-x-2">
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
        </div>
    );
}
