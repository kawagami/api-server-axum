"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Loader2, Trash2 } from "lucide-react";
import { deleteUser } from "./actions";

interface Props {
    user: { id: string; name: string };
    // 目前登入者本人那列 → 停用，避免誤刪自己導致無人可管
    isSelf?: boolean;
}

export default function DeleteUserButton({ user, isSelf }: Props) {
    const router = useRouter();
    const [isDeleting, setIsDeleting] = useState(false);

    if (isSelf) {
        return (
            <span
                className="text-xs text-neutral-400 dark:text-neutral-500"
                title="無法刪除目前登入的帳號"
            >
                本人
            </span>
        );
    }

    const handleDelete = async () => {
        if (isDeleting) return;
        if (!confirm(`確定要刪除管理員「${user.name}」嗎？此操作無法復原。`)) return;

        setIsDeleting(true);
        try {
            await deleteUser(user);
            router.refresh();
        } catch (err) {
            if ((err as { digest?: string }).digest?.startsWith("NEXT_REDIRECT")) throw err;
            alert("刪除失敗，請稍後再試。");
        } finally {
            setIsDeleting(false);
        }
    };

    return (
        <button
            onClick={handleDelete}
            disabled={isDeleting}
            className={`inline-flex items-center gap-1 px-3 py-2 text-xs font-medium rounded-lg text-white transition-colors ${
                isDeleting ? "bg-neutral-400 cursor-not-allowed" : "bg-red-500 hover:bg-red-600"
            }`}
        >
            {isDeleting ? <Loader2 size={14} className="animate-spin" /> : <Trash2 size={14} />}
            {isDeleting ? "刪除中..." : "刪除"}
        </button>
    );
}
