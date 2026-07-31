"use client";

import { useTranslations } from "next-intl";
import PageTitle from "@/components/page-title";

export default function PublicError({ error, reset }: { error: Error; reset: () => void }) {
    const t = useTranslations("Error");

    return (
        <div className="flex flex-col items-center justify-center min-h-[60svh] gap-4 text-center px-4">
            <PageTitle variant="hero" title={t("title")} description={t("description")} />
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
