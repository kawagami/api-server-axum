"use client";

import { useRouter } from "next/navigation";
import { useTransition } from "react";
import { Loader2, Plus } from "lucide-react";
import { v4 as uuidv4 } from 'uuid';

/**
 * 新增文章：前端先產 uuid，直接進編輯器（該 id 要到第一次存檔才會有 DB 紀錄）。
 *
 * 這裡刻意留成 button + router.push 而不是 `<Link>`：href 每次 render 都會是不同的 uuid，
 * 對「複製連結」「開新分頁」沒有意義（既有的編輯連結才需要那些，見清單的標題欄）。
 *
 * 編輯 / 刪除的按鈕已不在此檔 —— 標題本身就是編輯連結、刪除走清單的確認框，
 * 兩者都需要那一列的資料（標題），住在 blogs-client 裡。
 */
export function CreateButton({ label = "新增文章" }: { label?: string }) {
    const router = useRouter();
    const [isPending, startTransition] = useTransition();

    return (
        <button
            disabled={isPending}
            onClick={() => startTransition(() => router.push(`/admin/blogs/${uuidv4()}`))}
            className={`px-4 py-2 bg-primary-600 text-white text-sm font-medium rounded-lg shadow-md flex items-center gap-1.5 transition-colors ${isPending ? "opacity-60 cursor-not-allowed" : "hover:bg-primary-700"}`}
        >
            {isPending ? <Loader2 className="w-4 h-4 animate-spin" /> : <Plus className="w-4 h-4" />}
            {label}
        </button>
    );
}
