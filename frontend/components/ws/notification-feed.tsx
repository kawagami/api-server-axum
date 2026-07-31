"use client";

import { useWsContext } from "@/libs/ws-context";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import { useEffect, useState } from "react";
import { useLocale, useTranslations } from "next-intl";
import type { WsEventType } from "@/types";

const EVENT_TYPES: WsEventType[] = [
    'stock_completed',
    'stock_failed',
    'blog_created',
    'user_joined',
    'user_left',
    'admin_message',
];

interface FeedEntry {
    key: number;
    type: WsEventType;
    data: unknown;
    ts: string;
}

let seq = 0;

export default function NotificationFeed() {
    const { subscribe, unsubscribe } = useWsContext();
    const t = useTranslations("Ws");
    const locale = useLocale();
    const [entries, setEntries] = useState<FeedEntry[]>([]);

    useEffect(() => {
        // 時區釘死台北、locale 跟隨語系（裸 toLocaleTimeString 會跟著瀏覽器跑，同一份資料各人不同）
        const timeFmt = new Intl.DateTimeFormat(locale, {
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
            timeZone: "Asia/Taipei",
        });
        const handlers = EVENT_TYPES.map(type => {
            const fn = (data: unknown) => {
                setEntries(prev =>
                    [{ key: ++seq, type, data, ts: timeFmt.format(new Date()) }, ...prev].slice(0, 100)
                );
            };
            subscribe(type, fn);
            return { type, fn };
        });

        return () => {
            handlers.forEach(({ type, fn }) => unsubscribe(type, fn));
        };
    }, [subscribe, unsubscribe, locale]);

    return (
        <PageShell width="form" className="flex flex-col gap-3">
            <PageTitle title={t("feedTitle")} />
            {entries.length === 0 ? (
                <p className="text-neutral-500 dark:text-neutral-400 text-sm">{t("waiting")}</p>
            ) : (
                entries.map(entry => (
                    <div key={entry.key} className="bg-white dark:bg-neutral-800 rounded-lg p-4 shadow text-sm font-mono">
                        <div className="flex items-center gap-3 mb-1">
                            <span className="text-primary-600 dark:text-primary-400 font-semibold">{entry.type}</span>
                            <span className="text-neutral-400 text-xs">{entry.ts}</span>
                        </div>
                        <pre className="text-neutral-700 dark:text-neutral-300 whitespace-pre-wrap break-all text-xs">
                            {JSON.stringify(entry.data, null, 2)}
                        </pre>
                    </div>
                ))
            )}
        </PageShell>
    );
}
