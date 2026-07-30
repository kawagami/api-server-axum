"use client";

import { useEffect, useState } from "react";

export type ToastKind = "success" | "error";
export interface ToastState {
    kind: ToastKind;
    message: string;
}

/**
 * 前台一次性提示的單一來源。取代散在各頁的 `alert()` 與自己刻的置中覆蓋層
 * ——alert 會擋住整個視窗、無法翻譯樣式，覆蓋層則各頁位置不一。
 * 固定浮在畫面底部中央，role="status" 讓螢幕閱讀器也讀得到。
 */
export function useToast(timeoutMs = 2400) {
    const [toast, setToast] = useState<ToastState | null>(null);

    useEffect(() => {
        if (!toast) return;
        const timer = setTimeout(() => setToast(null), timeoutMs);
        return () => clearTimeout(timer);
    }, [toast, timeoutMs]);

    return {
        toast,
        showToast: (kind: ToastKind, message: string) => setToast({ kind, message }),
    };
}

export default function Toast({ toast }: { toast: ToastState | null }) {
    if (!toast) return null;

    return (
        <div
            role="status"
            aria-live="polite"
            className={`fixed bottom-6 left-1/2 z-50 -translate-x-1/2 rounded-lg px-4 py-2 text-sm text-white shadow-lg ${
                toast.kind === "success" ? "bg-primary-600" : "bg-red-500"
            }`}
        >
            {toast.message}
        </div>
    );
}
