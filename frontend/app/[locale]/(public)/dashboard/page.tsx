import { getCurrentMember } from "@/api/members";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import FeatureCard from "@/components/feature-card";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";
import { MEMBER_LINKS, filterNavByFeatures } from "@/libs/site-nav";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures } from "@/libs/enabled-features";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Dashboard");
    return { title: t("title") };
}

export default async function DashboardPage() {
    const [member, t, tHeader, settings] = await Promise.all([
        getCurrentMember(),
        getTranslations("Dashboard"),
        getTranslations("Header"),
        getPublicSettings(),
    ]);
    const enabled = resolveEnabledFeatures(settings.enabled_features);
    const memberLinks = filterNavByFeatures(MEMBER_LINKS, enabled);

    return (
        <PageShell className="flex flex-col gap-8">
            <div className="flex items-center gap-4">
                {member.avatar_url ? (
                    // eslint-disable-next-line @next/next/no-img-element
                    <img
                        src={member.avatar_url}
                        alt={member.name}
                        className="w-14 h-14 rounded-full object-cover"
                    />
                ) : (
                    <div className="w-14 h-14 rounded-full bg-primary-100 dark:bg-primary-900 flex items-center justify-center text-xl font-bold text-primary-600 dark:text-primary-300">
                        {member.name.charAt(0).toUpperCase()}
                    </div>
                )}
                <div>
                    <p className="text-sm text-neutral-500 dark:text-neutral-400">{t("welcomeBack")}</p>
                    <PageTitle title={member.name} />
                </div>
            </div>

            <section className="flex flex-col gap-3">
                <h2 className="text-sm font-semibold text-neutral-500 dark:text-neutral-400">{t("myFeatures")}</h2>
                <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {memberLinks.filter(({ key }) => key !== "dashboard").map(({ key, href, labelKey, icon }) => (
                        <FeatureCard
                            key={key}
                            href={href}
                            icon={icon}
                            title={tHeader(labelKey)}
                            desc={t(`items.${key}`)}
                        />
                    ))}
                </div>
            </section>
        </PageShell>
    );
}
