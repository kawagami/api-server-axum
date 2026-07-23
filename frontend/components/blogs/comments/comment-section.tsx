"use client";

import { useEffect, useState } from "react";
import { useTranslations } from "next-intl";
import { Loader2, MessageSquare, UserRound } from "lucide-react";
import ShowClientTime from "@/components/blogs/show-client-time";
import usePagedList from "@/hooks/usePagedList";
import { getBlogComments, postBlogComment } from "@/api/blog-comments";
import type { BlogComment } from "@/types";

const CONTENT_MAX = 5000;
const LIMIT = 50;

type ErrorKey = "empty" | "rateLimit" | "invalid" | "failed";

export default function CommentSection({ blogId, isMember }: { blogId: string; isMember: boolean }) {
    const t = useTranslations("BlogComments");
    const { items: comments, hasMore, isPending, load, loadMore, setItems } =
        usePagedList<BlogComment>(LIMIT);
    const [content, setContent] = useState("");
    const [name, setName] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [errorKey, setErrorKey] = useState<ErrorKey | null>(null);
    const [loaded, setLoaded] = useState(false);

    useEffect(() => {
        load(async page => {
            const data = await getBlogComments(blogId, page, LIMIT);
            setLoaded(true);
            return data;
        });
    }, [load, blogId]);

    async function handleSubmit(e: React.FormEvent) {
        e.preventDefault();
        if (submitting) return;
        const trimmed = content.trim();
        if (!trimmed) {
            setErrorKey("empty");
            return;
        }
        setSubmitting(true);
        setErrorKey(null);
        try {
            const created = await postBlogComment(blogId, {
                content: trimmed,
                name: isMember ? undefined : name.trim() || undefined,
            });
            setItems(prev => [...prev, created]);
            setContent("");
        } catch (err) {
            const msg = err instanceof Error ? err.message : "";
            if (msg.includes("429")) setErrorKey("rateLimit");
            else if (msg.includes("422")) setErrorKey("invalid");
            else setErrorKey("failed");
        } finally {
            setSubmitting(false);
        }
    }

    return (
        <section className="mt-12 border-t border-neutral-200 dark:border-neutral-700 pt-8">
            <h2 className="flex items-center gap-2 text-lg font-semibold text-neutral-800 dark:text-neutral-100 mb-6">
                <MessageSquare className="w-5 h-5" />
                {t("title")}
                {loaded && comments.length > 0 && (
                    <span className="text-sm font-normal text-neutral-500 dark:text-neutral-400">
                        ({comments.length})
                    </span>
                )}
            </h2>

            {/* 留言清單(舊到新) */}
            {!loaded ? (
                <div className="flex justify-center py-6 text-neutral-400">
                    <Loader2 className="w-5 h-5 animate-spin" />
                </div>
            ) : comments.length === 0 ? (
                <p className="text-sm text-neutral-500 dark:text-neutral-400 py-2">{t("empty")}</p>
            ) : (
                <ul className="flex flex-col gap-5">
                    {comments.map(c => (
                        <li key={c.id} className="flex gap-3">
                            <div className="shrink-0">
                                {c.is_member && c.avatar_url ? (
                                    // OAuth 頭像為外部 URL,無法經 next/image 最佳化
                                    // eslint-disable-next-line @next/next/no-img-element
                                    <img
                                        src={c.avatar_url}
                                        alt=""
                                        className="w-9 h-9 rounded-full object-cover"
                                    />
                                ) : (
                                    <div className="w-9 h-9 rounded-full bg-neutral-200 dark:bg-neutral-700 flex items-center justify-center text-neutral-500 dark:text-neutral-400">
                                        <UserRound className="w-5 h-5" />
                                    </div>
                                )}
                            </div>
                            <div className="min-w-0 flex-1">
                                <div className="flex items-center flex-wrap gap-x-2 gap-y-1">
                                    <span className="font-medium text-sm text-neutral-800 dark:text-neutral-100">
                                        {c.author_name || t("guest")}
                                    </span>
                                    {c.is_member && (
                                        <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300">
                                            {t("memberBadge")}
                                        </span>
                                    )}
                                    <span className="text-xs">
                                        <ShowClientTime datetimeString={c.created_at} />
                                    </span>
                                </div>
                                <p className="mt-1 text-sm text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap break-words">
                                    {c.content}
                                </p>
                            </div>
                        </li>
                    ))}
                </ul>
            )}

            {hasMore && (
                <div className="flex justify-center mt-5">
                    <button
                        onClick={loadMore}
                        disabled={isPending}
                        className="px-4 py-1.5 text-sm rounded-md border border-neutral-300 dark:border-neutral-600 text-neutral-600 dark:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 disabled:opacity-50 transition-colors"
                    >
                        {t("loadMore")}
                    </button>
                </div>
            )}

            {/* 留言表單 */}
            <form onSubmit={handleSubmit} className="mt-8 flex flex-col gap-3">
                {!isMember && (
                    <input
                        type="text"
                        value={name}
                        onChange={e => setName(e.target.value)}
                        maxLength={100}
                        placeholder={t("namePlaceholder")}
                        className="w-full sm:max-w-xs rounded-md border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 px-3 py-2 text-sm text-neutral-800 dark:text-neutral-100 placeholder-neutral-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 transition-colors"
                    />
                )}
                {isMember && (
                    <p className="text-xs text-neutral-500 dark:text-neutral-400">{t("asMember")}</p>
                )}
                <textarea
                    value={content}
                    onChange={e => setContent(e.target.value)}
                    maxLength={CONTENT_MAX}
                    rows={3}
                    placeholder={t("contentPlaceholder")}
                    className="w-full rounded-md border border-neutral-300 dark:border-neutral-600 bg-white dark:bg-neutral-800 px-3 py-2 text-sm text-neutral-800 dark:text-neutral-100 placeholder-neutral-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500 transition-colors resize-y"
                />
                {errorKey && (
                    <p className="text-sm text-red-600 dark:text-red-400">{t(`error.${errorKey}`)}</p>
                )}
                <div className="flex justify-end">
                    <button
                        type="submit"
                        disabled={submitting}
                        className="inline-flex items-center gap-2 px-5 py-2 rounded-md bg-primary-600 hover:bg-primary-700 text-white text-sm font-medium disabled:opacity-50 transition-colors"
                    >
                        {submitting && <Loader2 className="w-4 h-4 animate-spin" />}
                        {t("submit")}
                    </button>
                </div>
            </form>
        </section>
    );
}
