import Link from "next/link";
import { getTranslations } from "next-intl/server";

// locale 之外的最後防線（有 locale prefix 的 404 走 (public)/not-found.tsx）。
// 這裡沒有 locale segment，getTranslations 會落在 i18n/request.ts 的 defaultLocale，
// 但文案仍走 messages —— 不寫死中文，三語系共用同一份 key。
export default async function NotFound() {
    const t = await getTranslations("NotFound");

    return (
        <div className="flex flex-col items-center justify-center min-h-[80svh] gap-4 text-center px-4">
            <h1 className="text-6xl font-bold text-neutral-300 dark:text-neutral-600">404</h1>
            <p className="text-neutral-500 dark:text-neutral-400">{t("description")}</p>
            <Link href="/" className="px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition-colors">
                {t("backHome")}
            </Link>
        </div>
    );
}
