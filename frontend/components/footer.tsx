import NextLink from "next/link";
import { Link } from "@/i18n/navigation";
import { getTranslations } from "next-intl/server";
import { MessageSquare } from "lucide-react";
import GithubMark from "@/components/github-mark";
import { isFeatureEnabled } from "@/libs/enabled-features";

export default async function Footer({ enabledFeatures = null }: { enabledFeatures?: string[] | null }) {
    const t = await getTranslations("Footer");
    const showContact = isFeatureEnabled(enabledFeatures, "message");

    return (
        <footer className="min-h-[50px] flex items-center justify-center gap-4 text-center">
            {showContact && (
                <Link href="/contact" className="inline-flex items-center gap-1 text-sm text-neutral-600 dark:text-neutral-300 hover:text-primary-600 dark:hover:text-primary-400 hover:underline transition-colors">
                    <MessageSquare className="w-4 h-4" />
                    {t("contact")}
                </Link>
            )}
            <NextLink className="hover:scale-90" target="_blank" href="https://github.com/kawagami">
                <GithubMark className="text-neutral-900 dark:text-white" />
            </NextLink>
        </footer>
    );
}
