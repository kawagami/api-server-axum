"use client";

import { useRouter } from "next/navigation";
import { useState, useTransition, type ButtonHTMLAttributes } from "react";
import { Loader2 } from "lucide-react";
import { v4 as uuidv4 } from 'uuid';
import Toast, { useToast } from "@/components/toast";
import { deleteBlog } from "@/api/blogs";

interface LoadingButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    loading: boolean;
}

function LoadingButton({ loading, children, className = '', ...props }: LoadingButtonProps) {
    return (
        <button disabled={loading} className={className} {...props}>
            {loading && <Loader2 className="w-4 h-4 animate-spin" />}
            {children}
        </button>
    );
}

export function CreateButton() {
    const router = useRouter();
    const [isPending, startTransition] = useTransition();

    return (
        <LoadingButton
            loading={isPending}
            onClick={() => startTransition(() => router.push(`/admin/blogs/${uuidv4()}`))}
            className={`px-6 py-2 bg-primary-500 text-white font-semibold rounded-lg shadow-md flex items-center gap-2 ${isPending ? "opacity-60 cursor-not-allowed" : "hover:bg-primary-600"}`}
        >
            新增 Blog
        </LoadingButton>
    );
}

export function EditButton({ uuid }: { uuid: string }) {
    const router = useRouter();
    const [isPending, startTransition] = useTransition();

    return (
        <LoadingButton
            loading={isPending}
            onClick={() => startTransition(() => router.push(`/admin/blogs/${uuid}`))}
            className={`ml-2 px-4 py-2 bg-green-500 text-white font-medium rounded-lg flex items-center gap-2 transition duration-200 ${isPending ? "opacity-60 cursor-not-allowed" : "hover:bg-green-600"}`}
        >
            編輯
        </LoadingButton>
    );
}

export function DeleteButton({ uuid }: { uuid: string }) {
    const router = useRouter();
    const [isDeleting, setIsDeleting] = useState(false);
    // 這顆按鈕在清單列的行內動作列裡（flex space-x-2），塞不進 ErrorBanner 這種塊狀元素，
    // 故一次性失敗提示走 Toast（不用 window.alert）。頁面層級的失敗仍用 ErrorBanner。
    const { toast, showToast } = useToast();

    const handleDelete = async () => {
        if (isDeleting) return;
        const confirmed = confirm("確定要刪除這篇 blog 嗎？");
        if (!confirmed) return;

        setIsDeleting(true);
        try {
            await deleteBlog(uuid);
            router.refresh();
        } catch (err) {
            if ((err as { digest?: string }).digest?.startsWith('NEXT_REDIRECT')) throw err;
            showToast("error", "刪除失敗，請稍後再試");
        } finally {
            setIsDeleting(false);
        }
    };

    return (
        <>
            <LoadingButton
                loading={isDeleting}
                onClick={handleDelete}
                className={`ml-2 px-4 py-2 font-medium rounded-lg text-white flex items-center gap-1 transition duration-200 ${isDeleting
                    ? "bg-neutral-400 cursor-not-allowed"
                    : "bg-red-500 hover:bg-red-600"
                    }`}
            >
                {isDeleting ? "刪除中..." : "刪除"}
            </LoadingButton>
            <Toast toast={toast} />
        </>
    );
}
