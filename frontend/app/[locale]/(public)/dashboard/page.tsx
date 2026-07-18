import { getCurrentMember } from "@/api/members";
import { Link } from "@/i18n/navigation";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { ScanLine, Ticket } from "lucide-react";
import FeatureCard from "@/components/feature-card";
import { MEMBER_LINKS, filterNavByFeatures } from "@/libs/site-nav";
import { getPublicSettings } from "@/api/settings";
import { resolveEnabledFeatures, isFeatureEnabled } from "@/libs/enabled-features";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Dashboard");
    return { title: t("title") };
}

// 快速操作是 deep-link 捷徑（非導航），留在本頁維護，不進 site-nav
const QUICK_ACTIONS = [
    { href: "/invoices/scan", labelKey: "scanInvoice", descKey: "scanInvoiceDesc", icon: ScanLine, feature: "invoices" },
    { href: "/lotto/register", labelKey: "registerLotto", descKey: "registerLottoDesc", icon: Ticket, feature: "lotto" },
] as const;

export default async function DashboardPage() {
    const [member, t, tHeader, settings] = await Promise.all([
        getCurrentMember(),
        getTranslations("Dashboard"),
        getTranslations("Header"),
        getPublicSettings(),
    ]);
    const enabled = resolveEnabledFeatures(settings.enabled_features);
    const quickActions = QUICK_ACTIONS.filter(({ feature }) => isFeatureEnabled(enabled, feature));
    const memberLinks = filterNavByFeatures(MEMBER_LINKS, enabled);

    return (
        <div className="w-full max-w-3xl px-4 py-8 flex flex-col gap-8">
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
                    <h1 className="text-2xl font-bold">{member.name}</h1>
                </div>
            </div>

            <section className="flex flex-col gap-3">
                <h2 className="text-sm font-semibold text-neutral-500 dark:text-neutral-400">{t("quickActions")}</h2>
                <div className="flex flex-col sm:flex-row gap-3">
                    {quickActions.map(({ href, labelKey, descKey, icon: Icon }) => (
                        <Link
                            key={href}
                            href={href}
                            className="flex-1 flex items-center gap-3 bg-primary-500 hover:bg-primary-600 text-white rounded-xl px-5 py-4 shadow-md hover:shadow-lg transition-colors"
                        >
                            <Icon size={24} className="shrink-0" />
                            <span className="flex flex-col">
                                <span className="font-semibold">{t(labelKey)}</span>
                                <span className="text-xs text-primary-100">{t(descKey)}</span>
                            </span>
                        </Link>
                    ))}
                </div>
            </section>

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
        </div>
    );
}
