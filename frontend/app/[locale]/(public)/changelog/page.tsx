import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { getTranslations } from "next-intl/server";
import NextLink from "next/link";
import { Sparkles, Bug, Zap, Wrench, ShieldCheck, ExternalLink, type LucideIcon } from "lucide-react";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import GithubMark from "@/components/github-mark";
import { localeAlternates } from "@/libs/seo";
import { getChangelog, getChangelogRepo } from "@/api/github";
import { groupByDay, type ChangelogType } from "@/libs/changelog";

const TYPE_ICON: Record<ChangelogType, LucideIcon> = {
    feat: Sparkles,
    fix: Bug,
    perf: Zap,
    refactor: Wrench,
    security: ShieldCheck,
};

export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale, namespace: "Changelog" });

    return {
        title: t("metaTitle"),
        description: t("metaDescription"),
        alternates: localeAlternates(locale, "/changelog"),
        openGraph: {
            type: "website",
            title: t("metaTitle"),
            description: t("metaDescription"),
            url: `/${locale}/changelog`,
        },
    };
}

export default async function Changelog({ params }: { params: Promise<{ locale: string }> }) {
    const { locale } = await params;
    const repo = await getChangelogRepo();
    // GITHUB_REPO 設空字串 = 這個 instance 不提供更新紀錄，整頁不存在
    if (!repo) notFound();

    const [t, entries] = await Promise.all([
        getTranslations("Changelog"),
        getChangelog(),
    ]);

    // commit 時間用 UTC 分組（與 groupByDay 切的 ISO 前 10 碼一致），顯示也用 UTC，
    // 否則同一筆會因伺服器時區落在不同天。
    const dayFormat = new Intl.DateTimeFormat(locale, {
        year: "numeric",
        month: "long",
        day: "numeric",
        timeZone: "UTC",
    });
    const groups = groupByDay(entries);

    return (
        <PageShell className="flex flex-col gap-8">
            <PageTitle variant="hero" title={t("title")} description={t("subtitle")} />

            {groups.length === 0 ? (
                <div className="bg-white dark:bg-neutral-800 rounded-xl shadow-md p-6 sm:p-8 text-center text-neutral-600 dark:text-neutral-400">
                    {t("empty")}
                </div>
            ) : (
                <div className="flex flex-col gap-6">
                    {groups.map(({ day, items }) => (
                        <section key={day} className="flex flex-col gap-3">
                            <h2 className="text-sm font-semibold uppercase tracking-wide text-neutral-500 dark:text-neutral-400">
                                {dayFormat.format(new Date(`${day}T00:00:00Z`))}
                            </h2>
                            <ul className="flex flex-col gap-2">
                                {items.map((entry) => {
                                    const Icon = TYPE_ICON[entry.type];
                                    return (
                                        <li
                                            key={entry.sha}
                                            className="bg-white dark:bg-neutral-800 rounded-xl shadow-md p-4 flex flex-wrap items-start gap-3"
                                        >
                                            <span className="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium bg-primary-100 text-primary-700 dark:bg-primary-900/60 dark:text-primary-300 shrink-0">
                                                <Icon className="w-3.5 h-3.5" />
                                                {t(`types.${entry.type}`)}
                                            </span>
                                            <div className="min-w-0 flex-1">
                                                <p className="text-neutral-800 dark:text-neutral-100 break-words">
                                                    {entry.breaking && (
                                                        <span className="mr-1 font-semibold text-red-600 dark:text-red-400">
                                                            {t("breaking")}
                                                        </span>
                                                    )}
                                                    {entry.subject}
                                                </p>
                                                {entry.scope && (
                                                    <p className="mt-1 text-xs text-neutral-500 dark:text-neutral-400">
                                                        {entry.scope}
                                                    </p>
                                                )}
                                            </div>
                                            <NextLink
                                                href={entry.url}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="inline-flex items-center gap-1 text-xs font-mono text-neutral-500 dark:text-neutral-400 hover:text-primary-600 dark:hover:text-primary-400 transition-colors shrink-0"
                                            >
                                                {entry.sha}
                                                <ExternalLink className="w-3 h-3" />
                                            </NextLink>
                                        </li>
                                    );
                                })}
                            </ul>
                        </section>
                    ))}
                </div>
            )}

            <section className="flex flex-col items-center gap-3 text-center">
                <p className="text-xs text-neutral-500 dark:text-neutral-400 max-w-xl">
                    {t("footnote")}
                </p>
                <NextLink
                    href={`https://github.com/${repo}/commits`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-white dark:bg-neutral-800 shadow-md text-neutral-700 dark:text-neutral-200 hover:text-primary-600 dark:hover:text-primary-400 hover:shadow-lg transition-all"
                >
                    <GithubMark className="w-5 h-5 text-neutral-900 dark:text-white" />
                    {t("viewAll")}
                </NextLink>
            </section>
        </PageShell>
    );
}
