"use client";

import { useTranslations } from "next-intl";
import { Link } from "@/i18n/navigation";
import PageTitle from "@/components/page-title";

export default function BlogError({ error, reset }: { error: Error; reset: () => void }) {
    const t = useTranslations("BlogError");

    return (
        <div className="flex flex-col items-center justify-center min-h-[60svh] gap-4 text-center px-4">
            <PageTitle variant="hero" title={t("title")} description={t("message")} />
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
