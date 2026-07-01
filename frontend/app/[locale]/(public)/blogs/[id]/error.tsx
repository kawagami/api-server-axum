"use client";

import { useTranslations } from "next-intl";
import { Link } from "@/i18n/navigation";

export default function BlogError({ error, reset }: { error: Error; reset: () => void }) {
    const t = useTranslations("BlogError");

    return (
        <div className="flex flex-col items-center justify-center h-[calc(100svh-120px)] gap-4 text-center px-4">
            <h1 className="text-4xl font-bold text-neutral-700 dark:text-neutral-200">{t("title")}</h1>
            <p className="text-neutral-500 dark:text-neutral-400">{t("message")}</p>
            {process.env.NODE_ENV === 'development' && (
                <pre className="text-xs text-red-400 max-w-sm overflow-auto">{error.message}</pre>
            )}
            <div className="flex gap-3">
                <button
                    onClick={reset}
                    className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
                >
                    {t("retry")}
                </button>
                <Link href="/" className="px-4 py-2 bg-neutral-200 dark:bg-neutral-700 text-neutral-700 dark:text-neutral-200 rounded-lg hover:bg-neutral-300 dark:hover:bg-neutral-600 transition-colors">
                    {t("home")}
                </Link>
            </div>
        </div>
    );
}
