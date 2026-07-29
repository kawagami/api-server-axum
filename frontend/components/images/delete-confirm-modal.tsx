"use client";

import { useCallback } from "react";
import Image from "next/image";
import { Loader2 } from "lucide-react";
import useDialog from "@/hooks/useDialog";
import type { ManagedImage } from "@/hooks/useImageManager";

interface Props {
    image: ManagedImage;
    deleting: boolean;
    onConfirm: () => void;
    onClose: () => void;
}

export default function DeleteConfirmModal({ image, deleting, onConfirm, onClose }: Props) {
    // 刪除進行中不讓 Esc / 點背景關掉（避免以為取消了但請求已送出）
    const requestClose = useCallback(() => { if (!deleting) onClose(); }, [deleting, onClose]);
    const dialogRef = useDialog<HTMLDivElement>(true, requestClose);

    return (
        <div
            className="fixed inset-0 z-20 flex items-center justify-center bg-black/50 p-4"
            onClick={requestClose}
        >
            <div
                ref={dialogRef}
                role="dialog"
                aria-modal="true"
                aria-label="刪除圖片"
                className="bg-white dark:bg-neutral-800 p-6 rounded-lg shadow-lg w-full max-w-sm"
                onClick={(e) => e.stopPropagation()}
            >
                <h2 className="text-lg font-semibold mb-4">確定刪除此圖片？</h2>
                <div className="flex justify-center mb-4">
                    <Image
                        width={160}
                        height={160}
                        src={image.url}
                        alt="欲刪除的圖片"
                        className="rounded-lg object-cover max-h-40 w-auto ring-1 ring-neutral-200 dark:ring-neutral-700"
                    />
                </div>
                <p className="text-sm text-neutral-500 dark:text-neutral-400 mb-6 text-center">
                    此操作無法復原，已引用此圖片的內容將失效。
                </p>
                <div className="flex gap-2">
                    <button
                        onClick={onClose}
                        disabled={deleting}
                        className="flex-1 px-4 py-2 rounded-lg bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                    >
                        取消
                    </button>
                    <button
                        onClick={onConfirm}
                        disabled={deleting}
                        className="flex-1 flex items-center justify-center gap-1 px-4 py-2 rounded-lg bg-red-600 hover:bg-red-700 text-white disabled:bg-neutral-500 disabled:cursor-not-allowed transition-colors"
                    >
                        {deleting && <Loader2 className="w-4 h-4 animate-spin" />}
                        {deleting ? '刪除中…' : '刪除'}
                    </button>
                </div>
            </div>
        </div>
    );
}
