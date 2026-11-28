"use client";

import { useEffect, useState, type ButtonHTMLAttributes, type MouseEvent, type ReactNode } from "react";
import { useTranslations } from "next-intl";
import { Loader2 } from "lucide-react";
import { cn } from "@/libs/cn";

/** 送出後超過這麼久還沒回應，就把按鈕文字換成「網路較慢，仍在處理…」 */
const SLOW_MS = 8000;

type Props = Omit<ButtonHTMLAttributes<HTMLButtonElement>, "aria-busy" | "aria-disabled"> & {
    /** 動作進行中：轉圈、擋重複點擊（含表單 Enter 的隱式送出） */
    pending: boolean;
    /** 進行中顯示的文字（如「儲存中…」）；沒給就沿用 children */
    pendingLabel?: ReactNode;
    /** 超過 8 秒仍在進行時顯示的文字 */
    slowLabel?: string;
};

/**
 * 主動作按鈕（送出／儲存／刪除確認）的 pending 狀態。只用在「使用者剛點的那一顆」，
 * 同一表單裡跟著停用的次要按鈕（取消、上下移）照舊用 `disabled`。
 *
 * 為什麼不用 `disabled`：停用的按鈕會失去焦點、螢幕閱讀器也跳過它，鍵盤使用者按完 Enter
 * 焦點直接掉到 body，而且灰掉的按鈕在慢網路下看起來像「沒點到」。這裡改成
 * `aria-disabled` + `aria-busy`，按鈕保持可聚焦，重複點擊在 onClick 擋掉。
 *
 * 版面 class（`inline-flex items-center gap-*`）由呼叫端給 —— `libs/cn.ts` 不處理 Tailwind 衝突，
 * 元件塞版面 class 會和呼叫端的 `flex` / `w-full` 打架。
 *
 * 後台（沒有 next-intl）用這支，慢網路文案預設中文；前台用 `PublicPendingButton`。
 */
export default function PendingButton({
    pending,
    pendingLabel,
    slowLabel = "網路較慢，仍在處理…",
    onClick,
    className,
    type = "button",
    children,
    ...rest
}: Props) {
    const slow = useSlow(pending);

    function handleClick(e: MouseEvent<HTMLButtonElement>) {
        // 表單裡按 Enter 的隱式送出也會觸發 submit 按鈕的 click，preventDefault 一起擋掉
        if (pending) {
            e.preventDefault();
            return;
        }
        onClick?.(e);
    }

    return (
        <button
            {...rest}
            type={type}
            onClick={handleClick}
            aria-disabled={pending || undefined}
            aria-busy={pending || undefined}
            className={cn(className, "aria-busy:cursor-progress")}
        >
            {pending && <Loader2 className="size-4 shrink-0 animate-spin" aria-hidden />}
            {pending ? (slow ? slowLabel : (pendingLabel ?? children)) : children}
        </button>
    );
}

/** 前台版：慢網路文案走 i18n（`Common.slow`） */
export function PublicPendingButton(props: Omit<Props, "slowLabel">) {
    const t = useTranslations("Common");
    return <PendingButton {...props} slowLabel={t("slow")} />;
}

function useSlow(pending: boolean): boolean {
    const [slow, setSlow] = useState(false);
    useEffect(() => {
        if (!pending) return;
        const id = window.setTimeout(() => setSlow(true), SLOW_MS);
        return () => {
            window.clearTimeout(id);
            setSlow(false);
        };
    }, [pending]);
    return pending && slow;
}
