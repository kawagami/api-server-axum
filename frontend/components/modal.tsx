"use client";

import { useCallback } from "react";
import useDialog from "@/hooks/useDialog";
import { cn } from "@/libs/cn";

/**
 * 置中對話框的外殼（背景遮罩 + 面板 + a11y），取代各頁手寫的 `fixed inset-0` 殼。
 *
 * 收斂前有 15 處手寫遮罩，其中 7 處沒接 `useDialog` —— 少了 Esc 關閉、焦點鎖在框內、
 * 背景捲動鎖與關閉後還原焦點。那是 a11y 缺陷，不只是重複。
 *
 * 呼叫端自行條件渲染（`{open && <Modal …>}`），本元件永遠是開啟狀態。
 *
 * **不收**非置中的浮層：抽屜（admin sidebar / logs trace）、頂部下拉（header 手機選單）、
 * 命令面板（貼齊上方）、遊戲結局遮罩（不可關閉、沒有 onClose 語意）。硬塞進來只會讓
 * props 長成另一套 CSS。
 */

const SIZES = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
    xl: "max-w-2xl",
} as const;

/** 前後台卡片底色不同，面板要跟著所在區域走，否則深色模式會差一階 */
const SURFACES = {
    admin: "bg-white dark:bg-neutral-900",
    public: "bg-white dark:bg-neutral-800",
} as const;

const BACKDROPS = {
    dim: "bg-black/50",
    blur: "bg-neutral-900/70 backdrop-blur-xs",
} as const;

export default function Modal({
    label,
    onClose,
    dismissible = true,
    size = "sm",
    surface = "admin",
    backdrop = "dim",
    className = "",
    children,
}: {
    /** 給 `aria-label`；面板內通常同時有一個可見的 h2 */
    label: string;
    onClose: () => void;
    /** false = Esc 與點背景都不關（請求進行中，避免以為取消了但已送出） */
    dismissible?: boolean;
    size?: keyof typeof SIZES;
    surface?: keyof typeof SURFACES;
    backdrop?: keyof typeof BACKDROPS;
    /** 面板的內距／捲動／排版（`p-6`、`max-h-[80vh] overflow-auto`…）。
     *  底色、圓角、陰影、寬度由本元件給，不要在這裡覆寫（`cn` 不做 Tailwind 衝突合併）。 */
    className?: string;
    children: React.ReactNode;
}) {
    const requestClose = useCallback(() => { if (dismissible) onClose(); }, [dismissible, onClose]);
    const dialogRef = useDialog<HTMLDivElement>(true, requestClose);

    return (
        <div
            className={cn("fixed inset-0 z-50 flex items-center justify-center p-4", BACKDROPS[backdrop])}
            onClick={requestClose}
        >
            <div
                ref={dialogRef}
                role="dialog"
                aria-modal="true"
                aria-label={label}
                className={cn("w-full rounded-lg shadow-xl", SIZES[size], SURFACES[surface], className)}
                onClick={e => e.stopPropagation()}
            >
                {children}
            </div>
        </div>
    );
}
