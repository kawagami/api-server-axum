import { Suspense } from "react";
import { getTranslations } from "next-intl/server";
import KawaLogo from "@/components/kawa-logo";
import FeatureCard from "@/components/feature-card";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import LatestPosts from "./latest-posts";
import { getPublicSettings } from "@/api/settings";
import { resolveHomeFeatures } from "@/libs/home-features";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";

// 卡片網格欄數跟著卡片數走：instance 關掉功能後只剩 1~2 張時不要被拉成超寬卡片，
// 剛好 4 張時走 2x2 而不是 3+1 的孤兒列。
function gridClass(count: number): string {
    if (count <= 1) return "max-w-sm mx-auto";
    if (count === 2) return "sm:grid-cols-2 max-w-3xl mx-auto";
    if (count === 4) return "sm:grid-cols-2 max-w-3xl mx-auto";
    return "sm:grid-cols-2 lg:grid-cols-3";
}

export default async function ProjectIntro() {
    const [t, settings] = await Promise.all([
        getTranslations("Home"),
        getPublicSettings(),
    ]);
    // 卡片清單 = home_features（顯示+排序）∩ enabled_features（instance 功能開關）
    const enabled = resolveEnabledFeatures(settings.enabled_features);
    const features = resolveHomeFeatures(settings.home_features)
        .filter((f) => isFeatureEnabled(enabled, f.feature));
    const showBlog = isFeatureEnabled(enabled, "blog");

    return (
        <PageShell width="wide" className="flex flex-col gap-10">
            {/* Hero */}
            <section className="flex flex-col items-center gap-4 pt-2 sm:pt-6">
                <KawaLogo width={160} height={64} />
                <PageTitle variant="hero" title={t("title")} description={t("tagline")} />
                <p className="max-w-2xl mx-auto text-center text-xs sm:text-sm text-neutral-500 dark:text-neutral-400">
                    {t("techStack")}
                </p>
            </section>

            {/* 功能卡片 */}
            {features.length > 0 && (
                <section className={`grid grid-cols-1 gap-4 ${gridClass(features.length)}`}>
                    {features.map(({ key, href, icon }) => (
                        <FeatureCard
                            key={key}
                            href={href}
                            icon={icon}
                            title={t(`features.${key}.title`)}
                            desc={t(`features.${key}.desc`)}
                        />
                    ))}
                </section>
            )}

            {/* 最新文章：blog 功能關閉的 instance 不顯示；用 Suspense 讓 hero 先出來 */}
            {showBlog && (
                <Suspense fallback={<LatestPostsSkeleton />}>
                    <LatestPosts />
                </Suspense>
            )}
        </PageShell>
    );
}

function LatestPostsSkeleton() {
    return (
        <section className="flex flex-col gap-4">
            <div className="h-7 w-28 rounded-md bg-neutral-200 dark:bg-neutral-700 animate-pulse" />
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {[0, 1, 2].map((i) => (
                    <div key={i} className="h-36 rounded-xl bg-white dark:bg-neutral-800 shadow-md animate-pulse" />
                ))}
            </div>
        </section>
    );
}
