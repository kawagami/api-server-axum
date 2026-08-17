"use client";

import { useCallback } from "react";
import { Loader2 } from "lucide-react";
import useDialog from "@/hooks/useDialog";

/**
 * 後台破壞性操作的統一確認框，取代 `window.confirm`。
 *
 * 為什麼不用 `window.confirm`：訊息只能是純字串（塞不進「哪一筆」的視覺強調）、
 * 樣式不受控、部分瀏覽器設定可停用原生對話框 —— 停用時 `confirm()` 直接回 `false`，
 * 使用者按刪除完全沒反應，而且沒有任何徵兆。
 *
 * 呼叫端自行條件渲染（`{target && <ConfirmDialog … />}`），本元件永遠是開啟狀態。
 * Esc / 點背景關閉、焦點鎖在框內、關閉還原焦點都由 `useDialog` 提供。
 */
export default function ConfirmDialog({
    title,
    children,
    confirmLabel = "確定",
    busyLabel,
    busy = false,
    onConfirm,
    onClose,
}: {
    title: string;
    children?: React.ReactNode;
    confirmLabel?: string;
    busyLabel?: string;
    busy?: boolean;
    onConfirm: () => void;
    onClose: () => void;
}) {
    // 進行中不讓 Esc / 點背景關掉（避免以為取消了但請求已送出）
    const requestClose = useCallback(() => { if (!busy) onClose(); }, [busy, onClose]);
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
                aria-label={title}
                className="bg-white dark:bg-neutral-800 p-6 rounded-lg shadow-lg w-full max-w-sm"
                onClick={e => e.stopPropagation()}
            >
                <h2 className="text-lg font-semibold text-neutral-900 dark:text-neutral-100 mb-2">{title}</h2>
                <div className="text-sm text-neutral-600 dark:text-neutral-300 mb-6 wrap-break-word">
                    {children}
                </div>
                <div className="flex gap-2">
                    <button
                        onClick={onClose}
                        disabled={busy}
                        className="flex-1 px-4 py-2 rounded-lg bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-200 hover:bg-neutral-300 dark:hover:bg-neutral-600 disabled:opacity-50 transition-colors"
                    >
                        取消
                    </button>
                    <button
                        onClick={onConfirm}
                        disabled={busy}
                        className="flex-1 flex items-center justify-center gap-1 px-4 py-2 rounded-lg bg-red-600 hover:bg-red-700 text-white disabled:bg-neutral-500 disabled:cursor-not-allowed transition-colors"
                    >
                        {busy && <Loader2 className="w-4 h-4 animate-spin" />}
                        {busy ? (busyLabel ?? `${confirmLabel}中…`) : confirmLabel}
                    </button>
                </div>
            </div>
        </div>
    );
}
