"use client";

import Link from "next/link";
import { useLocale, useTranslations } from "next-intl";
import type { HackmdNote } from "@/types";

interface Props {
    notes: HackmdNote[];
    openArray: string[];
}

export default function HackMDNotesComponent({ notes, openArray }: Props) {
    const t = useTranslations("Hackmd");
    const locale = useLocale();
    const data = notes.filter(note => note.tags.some(tag => openArray.includes(tag)));

    // 固定 timeZone 讓 server/client 格式一致（不必 suppressHydrationWarning），locale 跟隨當前語系
    const dateFormatter = new Intl.DateTimeFormat(locale, {
        year: "numeric", month: "2-digit", day: "2-digit",
        hour: "2-digit", minute: "2-digit", second: "2-digit",
        timeZone: "Asia/Taipei",
    });

    return (
        <>
            <div className="text-lg font-semibold mb-4 text-neutral-800 dark:text-neutral-100">
                {t("count", { count: data.length })}
            </div>
            {data.length === 0 ? (
                <div className="text-neutral-500 dark:text-neutral-400 py-12">{t("empty")}</div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6 p-4">
                    {data.map(note => (
                        <div key={note.id} className="border border-neutral-300 dark:border-neutral-700 rounded-lg shadow-lg hover:shadow-xl transition-shadow bg-white dark:bg-neutral-800">
                            <Link className="block p-4 text-center" target="_blank" href={note.publish_link}>
                                <div className="text-2xl font-bold text-primary-600 dark:text-primary-400 mb-2">{note.title}</div>
                                <div className="text-neutral-600 dark:text-neutral-300">
                                    {dateFormatter.format(new Date(note.last_changed_at))}
                                </div>
                                <div className="flex gap-2 flex-wrap justify-center mt-2">
                                    {note.tags.map(category => (
                                        <span key={category} className="bg-neutral-200 dark:bg-neutral-700 text-sm px-3 py-1 rounded-full">
                                            {category}
                                        </span>
                                    ))}
                                </div>
                            </Link>
                        </div>
                    ))}
                </div>
            )}
        </>
    );
}
