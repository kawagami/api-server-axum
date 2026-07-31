import { getTranslations } from "next-intl/server";
import KawaLogo from "@/components/kawa-logo";
import FeatureCard from "@/components/feature-card";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import { getPublicSettings } from "@/api/settings";
import { resolveHomeFeatures } from "@/libs/home-features";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";

export default async function ProjectIntro() {
    const [t, settings] = await Promise.all([
        getTranslations("Home"),
        getPublicSettings(),
    ]);
    // 卡片清單 = home_features（顯示+排序）∩ enabled_features（instance 功能開關）
    const enabled = resolveEnabledFeatures(settings.enabled_features);
    const features = resolveHomeFeatures(settings.home_features)
        .filter((f) => isFeatureEnabled(enabled, f.feature));

    return (
        <PageShell width="wide" className="flex flex-col gap-10">
            {/* Hero */}
            <section className="flex flex-col items-center gap-4">
                <KawaLogo width={160} height={64} />
                <PageTitle variant="hero" title={t("title")} description={t("tagline")} />
                <p className="max-w-2xl mx-auto text-center text-xs sm:text-sm text-neutral-500 dark:text-neutral-400">
                    {t("techStack")}
                </p>
            </section>

            {/* 功能卡片 */}
            <section className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
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
        </PageShell>
    );
}
