"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { Trash2, ExternalLink } from "lucide-react";
import { getAllBlogComments, deleteBlogComment } from "@/api/blog-comments";
import { AdminTable, AdminHeadRow, AdminRow, AdminTh, AdminTd } from "@/components/admin/table";
import usePagedList from "@/hooks/usePagedList";
import type { BlogComment } from "@/types";

const LIMIT = 50;

const fmt = new Intl.DateTimeFormat("zh-TW", {
    year: "numeric", month: "2-digit", day: "2-digit",
    hour: "2-digit", minute: "2-digit", timeZone: "Asia/Taipei",
});

export default function BlogCommentsClient({ canDelete }: { canDelete: boolean }) {
    const { items: comments, hasMore, isPending, load, loadMore, setItems } =
        usePagedList<BlogComment>(LIMIT);
    const [deletingId, setDeletingId] = useState<number | null>(null);

    useEffect(() => {
        load(page => getAllBlogComments(page, LIMIT));
    }, [load]);

    async function handleDelete(id: number) {
        if (deletingId) return;
        if (!window.confirm("確定要刪除這則留言嗎?")) return;
        setDeletingId(id);
        try {
            await deleteBlogComment(id);
            setItems(prev => prev.filter(c => c.id !== id));
        } catch {
            window.alert("刪除失敗,請稍後再試");
        } finally {
            setDeletingId(null);
        }
    }

    const colSpan = canDelete ? 5 : 4;

    return (
        <div className="w-full flex flex-col gap-4">
            <h1 className="text-xl font-semibold text-neutral-800 dark:text-neutral-100">部落格留言</h1>

            <div className={`bg-white dark:bg-neutral-900 shadow-lg rounded-lg overflow-hidden transition-opacity ${isPending ? 'opacity-60' : ''}`}>
                <div className="overflow-x-auto">
                    <AdminTable className="text-sm">
                        <thead>
                            <AdminHeadRow>
                                <AdminTh className="whitespace-nowrap">時間</AdminTh>
                                <AdminTh className="whitespace-nowrap">來自</AdminTh>
                                <AdminTh className="min-w-[16rem]">內容</AdminTh>
                                <AdminTh className="whitespace-nowrap">文章</AdminTh>
                                {canDelete && <AdminTh className="whitespace-nowrap"></AdminTh>}
                            </AdminHeadRow>
                        </thead>
                        <tbody>
                            {comments.length === 0 ? (
                                <tr>
                                    <td colSpan={colSpan} className="px-4 py-8 text-center text-neutral-500 dark:text-neutral-400">
                                        {isPending ? '載入中…' : '目前沒有留言'}
                                    </td>
                                </tr>
                            ) : (
                                comments.map(c => (
                                    <AdminRow key={c.id}>
                                        <AdminTd className="whitespace-nowrap text-xs text-neutral-500 dark:text-neutral-400">
                                            {fmt.format(new Date(c.created_at))}
                                        </AdminTd>
                                        <AdminTd className="text-xs">
                                            <div className="flex items-center gap-1.5">
                                                <span className="text-neutral-800 dark:text-neutral-200">{c.author_name || '訪客'}</span>
                                                {c.is_member && (
                                                    <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300">
                                                        會員
                                                    </span>
                                                )}
                                            </div>
                                        </AdminTd>
                                        <AdminTd className="max-w-md whitespace-pre-wrap break-words text-neutral-800 dark:text-neutral-200">
                                            {c.content}
                                        </AdminTd>
                                        <AdminTd className="whitespace-nowrap">
                                            <Link
                                                href={`/zh-TW/blogs/${c.blog_id}`}
                                                target="_blank"
                                                className="inline-flex items-center gap-1 text-xs text-primary-700 dark:text-primary-300 hover:underline"
                                            >
                                                <ExternalLink className="w-3 h-3" />
                                                查看
                                            </Link>
                                        </AdminTd>
                                        {canDelete && (
                                            <AdminTd className="whitespace-nowrap">
                                                <button
                                                    onClick={() => handleDelete(c.id)}
                                                    disabled={deletingId === c.id}
                                                    className="inline-flex items-center gap-1 px-2 py-1 text-xs rounded text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 disabled:opacity-50 transition-colors"
                                                >
                                                    <Trash2 className="w-3.5 h-3.5" />
                                                    刪除
                                                </button>
                                            </AdminTd>
                                        )}
                                    </AdminRow>
                                ))
                            )}
                        </tbody>
                    </AdminTable>
                </div>
            </div>

            {hasMore && (
                <div className="flex justify-center pb-4">
                    <button
                        onClick={loadMore}
                        disabled={isPending}
                        className="px-6 py-2 bg-neutral-800 dark:bg-neutral-200 text-white dark:text-neutral-900 rounded hover:bg-neutral-700 dark:hover:bg-neutral-300 disabled:opacity-50 text-sm font-medium transition-colors"
                    >
                        {isPending ? '載入中…' : '載入更多'}
                    </button>
                </div>
            )}
        </div>
    );
}
