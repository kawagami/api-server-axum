import { getBlogs, getBlogTagCounts } from '@/api/blogs';
import BlogListCard from '@/components/blogs/blog-list-card';
import TagFilterBar from '@/components/blogs/tag-filter-bar';
import Pagination from '@/components/blogs/pagination';
import BlogSearchBar from '@/components/blogs/blog-search-bar';
import BlogEmptyReset from '@/components/blogs/blog-empty-reset';
import { makeExcerpt } from '@/libs/blog-excerpt';
import { getTranslations } from 'next-intl/server';
import { Suspense } from 'react';

interface Props {
    selectedTag?: string | null
    page?: number
    /** 關鍵字搜尋（比對文章內容） */
    q?: string | null
    /** 排序：oldest = 舊到新；其餘 = 新到舊 */
    sort?: string | null
    /** 作者頁：只列此 admin（users.name）的文章，並顯示作者標題 */
    author?: string | null
}

const PER_PAGE = 10;

export default async function BlogList({ selectedTag = null, page = 1, q = null, sort = null, author = null }: Props) {
    const [{ data: blogs, total }, tags, t] = await Promise.all([
        getBlogs({ page, per_page: PER_PAGE, tag: selectedTag, author, q, sort }),
        getBlogTagCounts(),
        getTranslations('BlogList'),
    ]);

    const totalPages = Math.ceil(total / PER_PAGE);

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto">
            <h1 className="sr-only">{author ? t('authorHeading', { author }) : t('heading')}</h1>
            <div className="max-w-4xl mx-auto px-4">
                {author && (
                    <h2 className="text-center text-lg font-semibold text-neutral-700 dark:text-neutral-200 pt-4">
                        {t('authorHeading', { author })}
                    </h2>
                )}
                <div className="pt-4">
                    <Suspense>
                        <BlogSearchBar q={q ?? ''} sort={sort ?? ''} />
                    </Suspense>
                </div>
                <div className="flex gap-6 pt-3">
                    <div className="flex-1 min-w-0">
                        {tags.length > 0 && (
                            <div className="sm:hidden pt-1">
                                <TagFilterBar tags={tags} selectedTag={selectedTag} variant="bar" />
                            </div>
                        )}
                        {blogs.length === 0 ? (
                            <div className="text-center text-neutral-500 dark:text-neutral-400 py-16">
                                <p>{t('empty')}</p>
                                {(selectedTag || q) && <BlogEmptyReset />}
                            </div>
                        ) : (
                            blogs.map((blog) => (
                                <BlogListCard
                                    key={blog.id}
                                    id={blog.id}
                                    toc={blog.tocs[0] || '未命名 blog'}
                                    excerpt={makeExcerpt(blog.markdown ?? '', blog.tocs[0] ?? '')}
                                    tags={blog.tags || []}
                                    created_at={blog.created_at ?? ''}
                                    updated_at={blog.updated_at ?? ''}
                                    author_name={blog.author_name ?? null}
                                />
                            ))
                        )}
                        <Suspense>
                            <Pagination page={page} totalPages={totalPages} />
                        </Suspense>
                    </div>
                    <aside className="w-44 shrink-0 pt-1 hidden sm:block">
                        <Suspense>
                            <TagFilterBar tags={tags} selectedTag={selectedTag} />
                        </Suspense>
                    </aside>
                </div>
            </div>
        </div>
    );
}
