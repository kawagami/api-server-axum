"use client";

import { useLocale } from "next-intl";

interface Props {
    datetimeString: string;
    /** full = 日期＋時分秒（列表卡片）；date = 只到日（首頁最新文章等空間吃緊處） */
    variant?: "full" | "date";
    className?: string;
}

export default function ShowClientTime({ datetimeString, variant = "full", className }: Props) {
    const locale = useLocale();

    // 固定 timeZone 讓 server/client 格式一致，locale 跟隨當前語系
    const formatted = new Intl.DateTimeFormat(locale, {
        year: "numeric", month: "2-digit", day: "2-digit",
        ...(variant === "full"
            ? { hour: "2-digit" as const, minute: "2-digit" as const, second: "2-digit" as const }
            : {}),
        timeZone: "Asia/Taipei",
    }).format(new Date(datetimeString));

    return <span className={className ?? "text-neutral-500 italic"}>{formatted}</span>;
}
