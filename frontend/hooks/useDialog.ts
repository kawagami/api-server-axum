"use client";

import { useEffect, useRef } from "react";

const FOCUSABLE = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
].join(",");

/**
 * modal / drawer 的共用行為：Esc 關閉、開啟時鎖住背景捲動、焦點移進容器並困在裡面、
 * 關閉後把焦點還給原本的元素。
 *
 * 用法：把回傳的 ref 掛在對話框容器上，並自行補 role="dialog" aria-modal="true"。
 * 容器常駐 DOM（靠 transform 滑出）的 drawer 記得在關閉時加 `inert`，
 * 否則鍵盤仍可 Tab 進看不見的內容。
 */
export default function useDialog<T extends HTMLElement>(open: boolean, onClose: () => void) {
    const ref = useRef<T>(null);

    useEffect(() => {
        if (!open) return;
        const node = ref.current;
        const restoreTo = document.activeElement as HTMLElement | null;

        const focusables = () =>
            node
                ? Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE)).filter(
                      el => el.offsetWidth > 0 || el.offsetHeight > 0 || el === document.activeElement,
                  )
                : [];

        focusables()[0]?.focus();

        const onKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                onClose();
                return;
            }
            if (e.key !== "Tab") return;
            const items = focusables();
            if (items.length === 0) return;
            const first = items[0];
            const last = items[items.length - 1];
            const active = document.activeElement;
            if (e.shiftKey && (active === first || !node?.contains(active))) {
                e.preventDefault();
                last.focus();
            } else if (!e.shiftKey && active === last) {
                e.preventDefault();
                first.focus();
            }
        };

        document.addEventListener("keydown", onKeyDown);
        const prevOverflow = document.body.style.overflow;
        document.body.style.overflow = "hidden";

        return () => {
            document.removeEventListener("keydown", onKeyDown);
            document.body.style.overflow = prevOverflow;
            // 關閉後把焦點還回去；元素已被卸載（例如導航離開）時就放著讓瀏覽器處理
            if (restoreTo?.isConnected) restoreTo.focus();
        };
    }, [open, onClose]);

    return ref;
}
