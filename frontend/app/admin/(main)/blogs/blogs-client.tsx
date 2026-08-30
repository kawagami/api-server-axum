"use client";

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { ExternalLink, Trash2 } from "lucide-react";
import { getAdminBlogs, deleteBlog } from "@/api/blogs";
import { CreateButton } from "@/components/blogs/blog-action-buttons";
import TagManager from "@/components/blogs/tag-manager";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd, AdminEmptyRow } from "@/components/admin/table";
import AdminTableContainer from "@/components/admin/admin-table-container";
import ConfirmDialog from "@/components/admin/confirm-dialog";
import ErrorBanner, { LOAD_FAILED, DELETE_FAILED } from "@/components/admin/error-banner";
import PageHeader from "@/components/admin/page-header";
import usePagedList from "@/hooks/usePagedList";
import useFilterUrl from "@/hooks/useFilterUrl";
import { formatDateTime } from "@/libs/admin-datetime";
import type { AdminBlogListItem, TagCount } from "@/types";
import { ADMIN_FILTER_INPUT } from "@/libs/input-styles";

const LIMIT = 50;

interface Filters {
    q: string;
    tag: string;
    sort: string;
}

const defaultFilters: Filters = { q: '', tag: '', sort: '' };

/** 沒有任何 h1 的文章（草稿、只有內文）在清單裡全長一樣，補 id 前 8 碼才分得出是哪一篇 */
function blogTitle(blog: AdminBlogListItem): string {
    return blog.tocs[0]?.trim() || `未命名文章 · ${blog.id.slice(0, 8)}`;
}

export default function BlogsClient({
    tags,
    canCreate,
    canDelete,
    canManageTags,
}: {
    tags: TagCount[];
    canCreate: boolean;
    canDelete: boolean;
    canManageTags: boolean;
}) {
    const { items: blogs, setItems, total, hasMore, isPending, failed, load, loadMore } =
        usePagedList<AdminBlogListItem>();
    const { initial, write } = useFilterUrl(defaultFilters);
    const [filters, setFilters] = useState<Filters>(initial);
    // 樂觀刪除後 usePagedList 的 total 仍是 server 當時給的值，自己扣掉才不會顯示「共 51 篇」卻只有 50 列
    const [removed, setRemoved] = useState(0);
    const [target, setTarget] = useState<AdminBlogListItem | null>(null);
    const [deletingId, setDeletingId] = useState<string | null>(null);
    const [deleteError, setDeleteError] = useState<string | null>(null);

    const runQuery = useCallback((next: Filters) => {
        load(page => getAdminBlogs({ ...next, page, per_page: LIMIT }));
    }, [load]);

    useEffect(() => {
        // 初次載入沿用 URL 帶進來的條件（重新整理 / 貼連結不掉查詢）。
        // 這裡走 runQuery 而不是 reload：effect 內不可同步 setState（`removed` 本來就是 0）
        runQuery(initial);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [runQuery]);

    /** 換一組條件重查：`removed` 是針對「目前這份結果」的補正，換查詢就得歸零 */
    function reload(next: Filters) {
        setRemoved(0);
        runQuery(next);
    }

    function handleSearch() {
        if (isPending) return;
        write(filters);
        reload(filters);
    }

    function handleReset() {
        setFilters(defaultFilters);
        write(defaultFilters);
        reload(defaultFilters);
    }

    function pickTag(tag: string) {
        const next = { ...filters, tag };
        setFilters(next);
        write(next);
        reload(next);
    }

    async function handleDelete(blog: AdminBlogListItem) {
        if (deletingId) return;
        setDeleteError(null);
        setDeletingId(blog.id);
        try {
            await deleteBlog(blog.id);
            // 就地移除，不 router.refresh() 重抓整頁
            setItems(prev => prev.filter(b => b.id !== blog.id));
            setRemoved(n => n + 1);
            setTarget(null);
        } catch (err) {
            if ((err as { digest?: string }).digest?.startsWith('NEXT_REDIRECT')) throw err;
            setDeleteError(DELETE_FAILED);
        } finally {
            setDeletingId(null);
        }
    }

    const filtered = Boolean(filters.q || filters.tag || filters.sort);
    const shown = Math.max(total - removed, 0);

    return (
        <div className="flex min-h-0 flex-1 flex-col gap-4">
            <PageHeader
                title="文章"
                description={filtered ? `符合條件 ${shown} 篇` : `共 ${shown} 篇`}
                actions={canCreate ? <CreateButton /> : undefined}
            />

            {/* Filter bar */}
            <div className="flex shrink-0 flex-wrap gap-2 items-end bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 border border-neutral-200 dark:border-neutral-700">
                <div className="flex flex-col gap-1">
                    <label htmlFor="blog-q" className="text-xs text-neutral-500 dark:text-neutral-400">
                        關鍵字（標題與內文）
                    </label>
                    {/* maxLength 對齊後端 services::blogs::MAX_SEARCH_LEN（100 字） */}
                    <input
                        id="blog-q"
                        type="text"
                        maxLength={100}
                        value={filters.q}
                        onChange={e => setFilters(f => ({ ...f, q: e.target.value }))}
                        onKeyDown={e => e.key === 'Enter' && !e.nativeEvent.isComposing && handleSearch()}
                        placeholder="axum"
                        className={`${ADMIN_FILTER_INPUT} w-48`}
                    />
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="blog-tag" className="text-xs text-neutral-500 dark:text-neutral-400">Tag</label>
                    <select
                        id="blog-tag"
                        value={filters.tag}
                        onChange={e => setFilters(f => ({ ...f, tag: e.target.value }))}
                        className={`${ADMIN_FILTER_INPUT} w-40`}
                    >
                        <option value="">全部</option>
                        {tags.map(({ tag, count }) => (
                            <option key={tag} value={tag}>{tag}（{count}）</option>
                        ))}
                    </select>
                </div>
                <div className="flex flex-col gap-1">
                    <label htmlFor="blog-sort" className="text-xs text-neutral-500 dark:text-neutral-400">排序</label>
                    <select
                        id="blog-sort"
                        value={filters.sort}
                        onChange={e => setFilters(f => ({ ...f, sort: e.target.value }))}
                        className={`${ADMIN_FILTER_INPUT} w-36`}
                    >
                        <option value="">最新建立</option>
                        <option value="updated">最近更新</option>
                        <option value="oldest">最早建立</option>
                    </select>
                </div>
                <div className="flex gap-2">
                    <button
                        onClick={handleSearch}
                        disabled={isPending}
                        className="px-4 py-1.5 text-sm font-medium rounded-sm bg-primary-600 hover:bg-primary-700 text-white disabled:opacity-50 transition-colors"
                    >
                        搜尋
                    </button>
                    <button
                        onClick={handleReset}
                        disabled={isPending}
                        className="px-4 py-1.5 text-sm font-medium rounded-sm bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-300 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                    >
                        重設
                    </button>
                </div>
            </div>

            {canManageTags && (
                <div className="shrink-0">
                    {/* tag 改名/合併會改到每一列的 tag，故一併重載清單 */}
                    <TagManager tags={tags} onChanged={() => reload(filters)} />
                </div>
            )}

            <ErrorBanner message={failed ? LOAD_FAILED : deleteError} />

            <div className={`flex min-h-0 flex-1 flex-col transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                <AdminTableContainer stickyHead fill>
                    <AdminTable className="text-sm table-fixed">
                        <thead>
                            <AdminHeadRow>
                                <AdminTh>標題</AdminTh>
                                <AdminTh className="w-[16em] hidden lg:table-cell">Tag</AdminTh>
                                <AdminTh className="w-[11em] hidden xl:table-cell">建立</AdminTh>
                                <AdminTh className="w-[11em]">更新</AdminTh>
                                <AdminTh className="w-[6em]">操作</AdminTh>
                            </AdminHeadRow>
                        </thead>
                        <tbody>
                            {blogs.length === 0 ? (
                                <AdminEmptyRow colSpan={5}>
                                    {isPending ? '載入中…' : filtered ? (
                                        <>
                                            沒有符合條件的文章。
                                            <button onClick={handleReset} className="ml-1 text-primary-600 dark:text-primary-300 hover:underline">
                                                清除條件
                                            </button>
                                        </>
                                    ) : canCreate ? (
                                        <div className="flex flex-col items-center gap-3">
                                            <span>還沒有文章。</span>
                                            <CreateButton label="建立第一篇" />
                                        </div>
                                    ) : '目前沒有文章'}
                                </AdminEmptyRow>
                            ) : (
                                blogs.map(blog => (
                                    <AdminRow key={blog.id}>
                                        <AdminTd className="truncate">
                                            {/* 標題本身就是編輯連結：中鍵開新分頁、複製連結、hover 看網址都要靠真的 <a> */}
                                            <Link
                                                href={`/admin/blogs/${blog.id}`}
                                                title={blogTitle(blog)}
                                                className="font-medium text-neutral-800 dark:text-neutral-100 hover:text-primary-600 dark:hover:text-primary-300 hover:underline"
                                            >
                                                {blogTitle(blog)}
                                            </Link>
                                        </AdminTd>
                                        <AdminTd className="hidden lg:table-cell">
                                            <div className="flex flex-wrap gap-1">
                                                {blog.tags.length === 0 && <span className="text-xs text-neutral-400">—</span>}
                                                {blog.tags.map(tag => (
                                                    <button
                                                        key={tag}
                                                        onClick={() => pickTag(tag)}
                                                        title={`只看「${tag}」`}
                                                        className="px-1.5 py-0.5 rounded-sm text-[11px] bg-primary-50 dark:bg-primary-900/40 border border-primary-200 dark:border-primary-800 text-neutral-700 dark:text-neutral-200 hover:border-primary-400 transition-colors"
                                                    >
                                                        {tag}
                                                    </button>
                                                ))}
                                            </div>
                                        </AdminTd>
                                        <AdminTd className="whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400 hidden xl:table-cell">
                                            {formatDateTime(blog.created_at)}
                                        </AdminTd>
                                        <AdminTd className="whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400">
                                            {formatDateTime(blog.updated_at)}
                                        </AdminTd>
                                        <AdminTd className="whitespace-nowrap">
                                            <div className="flex items-center gap-1">
                                                <Link
                                                    href={`/zh-TW/blogs/${blog.id}`}
                                                    target="_blank"
                                                    title="在新分頁看公開頁"
                                                    aria-label={`查看公開頁：${blogTitle(blog)}`}
                                                    className="p-1.5 rounded-sm text-neutral-400 hover:text-primary-600 dark:hover:text-primary-300 transition-colors"
                                                >
                                                    <ExternalLink className="w-3.5 h-3.5" />
                                                </Link>
                                                {canDelete && (
                                                    <button
                                                        onClick={() => setTarget(blog)}
                                                        aria-label={`刪除 ${blogTitle(blog)}`}
                                                        className="p-1.5 rounded-sm text-neutral-400 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors"
                                                    >
                                                        <Trash2 className="w-3.5 h-3.5" />
                                                    </button>
                                                )}
                                            </div>
                                        </AdminTd>
                                    </AdminRow>
                                ))
                            )}
                        </tbody>
                    </AdminTable>
                </AdminTableContainer>
            </div>

            {hasMore && (
                <div className="flex shrink-0 justify-center">
                    <button
                        onClick={loadMore}
                        disabled={isPending}
                        className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded-sm hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                    >
                        {isPending ? '載入中…' : '載入更多'}
                    </button>
                </div>
            )}

            {target && (
                <ConfirmDialog
                    title="刪除這篇文章？"
                    confirmLabel="刪除"
                    busyLabel="刪除中…"
                    busy={deletingId === target.id}
                    onClose={() => setTarget(null)}
                    onConfirm={() => handleDelete(target)}
                >
                    將刪除「<span className="font-medium text-neutral-900 dark:text-neutral-100">{blogTitle(target)}</span>」
                    及其上傳的圖片，此操作無法復原。
                </ConfirmDialog>
            )}
        </div>
    );
}
