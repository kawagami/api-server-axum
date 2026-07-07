"use client";

import { useState, useTransition } from "react";
import { useRouter } from "next/navigation";
import { Loader2, Pencil, Trash2, Tags } from "lucide-react";
import { renameBlogTag, deleteBlogTag } from "@/api/blogs";
import type { TagCount } from "@/types";

// 後台全站 tag 管理：改名/合併、刪除。一般 admin 只影響自己的文章，super_admin 全站（後端 owner scope 決定）。
export default function TagManager({ tags }: { tags: TagCount[] }) {
    const router = useRouter();
    const [busy, setBusy] = useState<string | null>(null);
    const [, startTransition] = useTransition();

    async function run(tag: string, fn: () => Promise<number>, done: (n: number) => string) {
        setBusy(tag);
        try {
            const affected = await fn();
            startTransition(() => router.refresh());
            alert(done(affected));
        } catch (err) {
            if ((err as { digest?: string }).digest?.startsWith("NEXT_REDIRECT")) throw err;
            alert("操作失敗，請稍後再試。");
        } finally {
            setBusy(null);
        }
    }

    function handleRename(tag: string) {
        const to = window.prompt(`將「${tag}」改名或合併到：`, tag)?.trim();
        if (!to || to === tag) return;
        run(tag, () => renameBlogTag(tag, to), (n) => `已將 ${n} 篇文章的「${tag}」改為「${to}」`);
    }

    function handleDelete(tag: string) {
        if (!window.confirm(`確定要從所有文章移除「${tag}」這個 tag 嗎？`)) return;
        run(tag, () => deleteBlogTag(tag), (n) => `已從 ${n} 篇文章移除「${tag}」`);
    }

    if (tags.length === 0) return null;

    return (
        <div className="bg-white dark:bg-neutral-800 shadow rounded-lg p-6 mt-6">
            <h2 className="flex items-center gap-2 text-lg font-semibold text-neutral-800 dark:text-neutral-100 mb-1">
                <Tags size={18} /> 管理 Tag
            </h2>
            <p className="text-xs text-neutral-500 dark:text-neutral-400 mb-4">
                改名可用來合併重複 tag（例：把「rust」合併進「Rust」）；只會影響你有權限的文章。
            </p>
            <ul className="flex flex-wrap gap-2">
                {tags.map(({ tag, count }) => (
                    <li
                        key={tag}
                        className="flex items-center gap-1.5 bg-primary-50 dark:bg-primary-900/40 border border-primary-200 dark:border-primary-800 rounded-lg pl-3 pr-1.5 py-1"
                    >
                        <span className="text-sm text-neutral-700 dark:text-neutral-200">{tag}</span>
                        <span className="text-xs text-neutral-400 tabular-nums">{count}</span>
                        {busy === tag ? (
                            <Loader2 size={15} className="animate-spin text-neutral-400 ml-1" />
                        ) : (
                            <>
                                <button
                                    onClick={() => handleRename(tag)}
                                    aria-label={`改名 ${tag}`}
                                    className="ml-1 p-1 rounded text-neutral-400 hover:text-primary-600 dark:hover:text-primary-300 transition-colors"
                                >
                                    <Pencil size={14} />
                                </button>
                                <button
                                    onClick={() => handleDelete(tag)}
                                    aria-label={`刪除 ${tag}`}
                                    className="p-1 rounded text-neutral-400 hover:text-red-600 transition-colors"
                                >
                                    <Trash2 size={14} />
                                </button>
                            </>
                        )}
                    </li>
                ))}
            </ul>
        </div>
    );
}
