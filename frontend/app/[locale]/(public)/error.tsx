"use client";

import { useTranslations } from "next-intl";

export default function PublicError({ error, reset }: { error: Error; reset: () => void }) {
    const t = useTranslations("Error");

    return (
        <div className="flex flex-col items-center justify-center min-h-[60svh] gap-4 text-center px-4">
            <h1 className="text-3xl sm:text-4xl font-bold text-neutral-700 dark:text-neutral-200">{t("title")}</h1>
            <p className="text-neutral-500 dark:text-neutral-400">{t("description")}</p>
            {process.env.NODE_ENV === 'development' && (
                <pre className="text-xs text-red-400 max-w-sm overflow-auto">{error.message}</pre>
            )}
            <button
                onClick={reset}
                className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
            >
                {t("retry")}
            </button>
        </div>
    );
}
