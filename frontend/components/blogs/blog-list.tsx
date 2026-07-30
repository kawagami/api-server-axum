import { getBlogs, getBlogTagCounts } from '@/api/blogs';
import BlogListCard from '@/components/blogs/blog-list-card';
import TagFilterBar from '@/components/blogs/tag-filter-bar';
import Pagination from '@/components/blogs/pagination';
import BlogSearchBar from '@/components/blogs/blog-search-bar';
import BlogEmptyReset from '@/components/blogs/blog-empty-reset';
import PageShell from '@/components/page-shell';
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
        <PageShell className="flex flex-col gap-4">
            <h1 className="sr-only">{author ? t('authorHeading', { author }) : t('heading')}</h1>
            {author && (
                <h2 className="text-center text-lg font-semibold text-neutral-700 dark:text-neutral-200">
                    {t('authorHeading', { author })}
                </h2>
            )}
            <Suspense>
                <BlogSearchBar q={q ?? ''} sort={sort ?? ''} />
            </Suspense>
            <div className="flex gap-6">
                <div className="flex-1 min-w-0">
                    {tags.length > 0 && (
                        <div className="sm:hidden">
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
                {/* body 捲動 + header sticky 之後，標籤欄才能真的黏住（top = header 50px + 呼吸 16px） */}
                <aside className="hidden sm:block w-44 shrink-0 self-start sticky top-[66px] max-h-[calc(100svh-82px)] overflow-y-auto">
                    <Suspense>
                        <TagFilterBar tags={tags} selectedTag={selectedTag} />
                    </Suspense>
                </aside>
            </div>
        </PageShell>
    );
}
