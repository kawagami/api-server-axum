"use client";

import { useState, useTransition } from "react";
import { useRouter } from "next/navigation";
import { Pencil, Trash2, Tags } from "lucide-react";
import Toast, { useToast } from "@/components/toast";
import ConfirmDialog from "@/components/admin/confirm-dialog";
import TagRenameModal from "@/components/blogs/tag-rename-modal";
import { renameBlogTag, deleteBlogTag } from "@/api/blogs";
import type { TagCount } from "@/types";

// 後台全站 tag 管理：改名/合併、刪除。一般 admin 只影響自己的文章，super_admin 全站（後端 owner scope 決定）。
//
// 位置刻意在清單**上方**且預設收合：這個面板原本排在文章清單之後，一頁 50 篇的話
// 要捲過整份清單才看得到，等於實質不可達。
export default function TagManager({ tags, onChanged }: { tags: TagCount[]; onChanged?: () => void }) {
    const router = useRouter();
    const [busy, setBusy] = useState<string | null>(null);
    const [renaming, setRenaming] = useState<string | null>(null);
    const [deleting, setDeleting] = useState<string | null>(null);
    const [, startTransition] = useTransition();
    // 成功/失敗都是一次性提示（成功訊息 ErrorBanner 做不到），統一走 useToast，不用 window.alert
    const { toast, showToast } = useToast();

    async function run(tag: string, fn: () => Promise<number>, done: (n: number) => string) {
        setBusy(tag);
        try {
            const affected = await fn();
            // router.refresh 重抓 server 端的 tag 統計；onChanged 讓列表那側也跟著更新（每列的 tag 會變）
            startTransition(() => router.refresh());
            onChanged?.();
            setRenaming(null);
            setDeleting(null);
            showToast("success", done(affected));
        } catch (err) {
            if ((err as { digest?: string }).digest?.startsWith("NEXT_REDIRECT")) throw err;
            showToast("error", "操作失敗，請稍後再試");
        } finally {
            setBusy(null);
        }
    }

    if (tags.length === 0) return null;

    // modal 與 Toast 放在 <details> **外面**：details 關閉時會隱藏所有非 summary 的子元素，
    // 擺在裡面的話收合狀態下的提示會整個看不見
    return (
        <>
        <details className="bg-white dark:bg-neutral-900 shadow-lg rounded-lg">
            <summary className="flex items-center gap-2 cursor-pointer select-none p-4 text-sm font-medium text-neutral-800 dark:text-neutral-100">
                <Tags size={16} />
                管理 Tag
                <span className="text-xs font-normal text-neutral-500 dark:text-neutral-400">
                    {tags.length} 個 — 可改名、合併或移除
                </span>
            </summary>
            <div className="px-4 pb-4 sm:px-6 sm:pb-6">
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
                            <button
                                onClick={() => setRenaming(tag)}
                                aria-label={`改名 ${tag}`}
                                className="ml-1 p-1.5 rounded-sm text-neutral-400 hover:text-primary-600 dark:hover:text-primary-300 transition-colors"
                            >
                                <Pencil size={14} />
                            </button>
                            <button
                                onClick={() => setDeleting(tag)}
                                aria-label={`刪除 ${tag}`}
                                className="p-1.5 rounded-sm text-neutral-400 hover:text-red-600 transition-colors"
                            >
                                <Trash2 size={14} />
                            </button>
                        </li>
                    ))}
                </ul>
            </div>
        </details>

            {renaming && (
                <TagRenameModal
                    tag={renaming}
                    tags={tags}
                    busy={busy === renaming}
                    onClose={() => setRenaming(null)}
                    onConfirm={to =>
                        run(renaming, () => renameBlogTag(renaming, to), n => `已將 ${n} 篇文章的「${renaming}」改為「${to}」`)
                    }
                />
            )}

            {deleting && (
                <ConfirmDialog
                    title="移除這個 tag？"
                    confirmLabel="移除"
                    busyLabel="移除中…"
                    busy={busy === deleting}
                    onClose={() => setDeleting(null)}
                    onConfirm={() =>
                        run(deleting, () => deleteBlogTag(deleting), n => `已從 ${n} 篇文章移除「${deleting}」`)
                    }
                >
                    將從所有文章移除「<span className="font-medium text-neutral-900 dark:text-neutral-100">{deleting}</span>」
                    （{tags.find(t => t.tag === deleting)?.count ?? 0} 篇）。文章本身不會被刪除，但此操作無法復原。
                </ConfirmDialog>
            )}

            <Toast toast={toast} />
        </>
    );
}
