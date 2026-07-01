import { getBlogs, getBlogTags } from '@/api/blogs';
import BlogListCard from '@/components/blogs/blog-list-card';
import TagFilterBar from '@/components/blogs/tag-filter-bar';
import Pagination from '@/components/blogs/pagination';
import { getTranslations } from 'next-intl/server';
import { Suspense } from 'react';

interface Props {
    selectedTag?: string | null
    page?: number
}

const PER_PAGE = 10;

export default async function BlogList({ selectedTag = null, page = 1 }: Props) {
    const [{ data: blogs, total }, tags, t] = await Promise.all([
        getBlogs({ page, per_page: PER_PAGE, tag: selectedTag }),
        getBlogTags(),
        getTranslations('BlogList'),
    ]);

    const totalPages = Math.ceil(total / PER_PAGE);

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto">
            <h1 className="sr-only">{t('heading')}</h1>
            <div className="max-w-4xl mx-auto flex gap-6 px-4">
                <div className="flex-1 min-w-0">
                    {tags.length > 0 && (
                        <div className="sm:hidden px-4 pt-2">
                            <TagFilterBar tags={tags} selectedTag={selectedTag} variant="bar" />
                        </div>
                    )}
                    {blogs.length === 0 ? (
                        <div className="text-center text-neutral-500 dark:text-neutral-400 py-16">
                            {t('empty')}
                        </div>
                    ) : (
                        blogs.map((blog) => (
                            <BlogListCard
                                key={blog.id}
                                id={blog.id}
                                toc={blog.tocs[0] || '未命名 blog'}
                                tags={blog.tags || []}
                                created_at={blog.created_at ?? ''}
                                updated_at={blog.updated_at ?? ''}
                            />
                        ))
                    )}
                    <Suspense>
                        <Pagination page={page} totalPages={totalPages} />
                    </Suspense>
                </div>
                <aside className="w-40 shrink-0 pt-2 hidden sm:block">
                    <Suspense>
                        <TagFilterBar tags={tags} selectedTag={selectedTag} />
                    </Suspense>
                </aside>
            </div>
        </div>
    );
}
