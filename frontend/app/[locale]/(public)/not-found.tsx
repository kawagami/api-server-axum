import { getTranslations } from "next-intl/server";
import { Link } from "@/i18n/navigation";
import PageTitle from "@/components/page-title";

/**
 * locale 底下的 404（notFound() 與不存在的前台路徑都走這裡），有 Header / Footer 也有翻譯。
 * app/not-found.tsx 是 locale 之外（例如 /foo）的最後防線，那支拿不到語系 context。
 */
export default async function LocaleNotFound() {
    const t = await getTranslations("NotFound");

    return (
        <div className="flex flex-col items-center justify-center min-h-[60svh] gap-4 text-center px-4">
            {/* 標題用 default 字級（不是 hero）：上面那顆 text-6xl 的 404 才是視覺主體 */}
            <p className="text-6xl font-bold text-neutral-300 dark:text-neutral-600">404</p>
            <PageTitle title={t("title")} description={t("description")} />
            <Link
                href="/"
                className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors"
            >
                {t("backHome")}
            </Link>
        </div>
    );
}
