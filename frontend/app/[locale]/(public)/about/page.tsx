import type { Metadata } from "next";
import { getTranslations } from "next-intl/server";
import { localeAlternates } from "@/libs/seo";
import NextLink from "next/link";
import { Link } from "@/i18n/navigation";
import { ArrowRight, MessageSquare } from "lucide-react";
import KawaLogo from "@/components/kawa-logo";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import GithubMark from "@/components/github-mark";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";

export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale, namespace: "About" });

    return {
        title: t("metaTitle"),
        description: t("metaDescription"),
        alternates: localeAlternates(locale, "/about"),
        openGraph: {
            type: "website",
            title: t("metaTitle"),
            description: t("metaDescription"),
            url: `/${locale}/about`,
        },
    };
}

export default async function About() {
    const [t, settings] = await Promise.all([
        getTranslations("About"),
        getPublicSettings(),
    ]);
    const enabled = resolveEnabledFeatures(settings.enabled_features);
    const showContact = isFeatureEnabled(enabled, "message");

    const tech = t.raw("tech") as string[];
    const principles = t.raw("principles") as { title: string; desc: string }[];

    return (
        <PageShell className="flex flex-col gap-8">
            {/* Hero */}
            <section className="flex flex-col items-center gap-4">
                <KawaLogo width={160} height={64} />
                <PageTitle variant="hero" title={t("title")} description={t("tagline")} />
            </section>

            {/* 簡介 */}
            <section className="bg-white dark:bg-neutral-800 rounded-xl shadow-md p-6 sm:p-8">
                <p className="text-neutral-700 dark:text-neutral-300 leading-relaxed">
                    {t("intro")}
                </p>
            </section>

            {/* 技術棧 */}
            <section>
                <h2 className="text-sm font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400 mb-3">
                    {t("techHeading")}
                </h2>
                <div className="flex flex-wrap gap-2">
                    {tech.map((name) => (
                        <span
                            key={name}
                            className="px-3 py-1 rounded-full text-sm font-medium bg-primary-100 text-primary-700 dark:bg-primary-900/60 dark:text-primary-300"
                        >
                            {name}
                        </span>
                    ))}
                </div>
            </section>

            {/* 關於這個站 */}
            <section>
                <h2 className="text-sm font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400 mb-3">
                    {t("principlesHeading")}
                </h2>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    {principles.map((p) => (
                        <div key={p.title} className="bg-white dark:bg-neutral-800 rounded-xl shadow-md p-5">
                            <h3 className="font-semibold text-primary-700 dark:text-primary-300 mb-2">{p.title}</h3>
                            <p className="text-sm text-neutral-600 dark:text-neutral-400 leading-relaxed">{p.desc}</p>
                        </div>
                    ))}
                </div>
            </section>

            {/* 連結 */}
            <section>
                <h2 className="text-sm font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400 mb-3">
                    {t("linksHeading")}
                </h2>
                <div className="flex flex-wrap items-center gap-3">
                    <NextLink
                        href="https://github.com/kawagami"
                        target="_blank"
                        className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-white dark:bg-neutral-800 shadow-md text-neutral-700 dark:text-neutral-200 hover:text-primary-600 dark:hover:text-primary-400 hover:shadow-lg transition-all"
                    >
                        <GithubMark className="w-5 h-5 text-neutral-900 dark:text-white" />
                        {t("github")}
                    </NextLink>
                    {showContact && (
                        <Link
                            href="/contact"
                            className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-white dark:bg-neutral-800 shadow-md text-neutral-700 dark:text-neutral-200 hover:text-primary-600 dark:hover:text-primary-400 hover:shadow-lg transition-all"
                        >
                            <MessageSquare className="w-5 h-5" />
                            {t("contact")}
                        </Link>
                    )}
                    <Link
                        href="/"
                        className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary-600 text-white shadow-md hover:bg-primary-700 hover:shadow-lg transition-all"
                    >
                        {t("home")}
                        <ArrowRight className="w-4 h-4" />
                    </Link>
                </div>
            </section>
        </PageShell>
    );
}
