"use client";

import { useLocale } from "next-intl";

export default function ShowClientTime({ datetimeString }: { datetimeString: string }) {
    const locale = useLocale();

    // 固定 timeZone 讓 server/client 格式一致，locale 跟隨當前語系
    const formatted = new Intl.DateTimeFormat(locale, {
        year: "numeric", month: "2-digit", day: "2-digit",
        hour: "2-digit", minute: "2-digit", second: "2-digit",
        timeZone: "Asia/Taipei",
    }).format(new Date(datetimeString));

    return <span className="text-neutral-500 italic">{formatted}</span>;
}
