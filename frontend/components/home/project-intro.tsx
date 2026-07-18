import { getTranslations } from "next-intl/server";
import KawaLogo from "@/components/kawa-logo";
import FeatureCard from "@/components/feature-card";
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
        <div className="w-full h-[calc(100svh-120px)] overflow-auto">
            <div className="max-w-5xl mx-auto px-4 pb-12">
                {/* Hero */}
                <section className="text-center pt-4 pb-10">
                    <div className="flex justify-center mb-4">
                        <KawaLogo width={160} height={64} />
                    </div>
                    <h1 className="text-3xl sm:text-4xl font-bold text-neutral-800 dark:text-neutral-100 mb-3">
                        {t("title")}
                    </h1>
                    <p className="max-w-2xl mx-auto text-neutral-600 dark:text-neutral-300 text-base sm:text-lg mb-3">
                        {t("tagline")}
                    </p>
                    <p className="max-w-2xl mx-auto text-xs sm:text-sm text-neutral-500 dark:text-neutral-400">
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
            </div>
        </div>
    );
}
