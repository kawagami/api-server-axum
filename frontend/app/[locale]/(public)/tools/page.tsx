import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import { localeAlternates } from '@/libs/seo';
import FeatureCard from '@/components/feature-card';
import PageShell from '@/components/page-shell';
import PageTitle from '@/components/page-title';
import { TOOLS, filterNavByFeatures } from '@/libs/site-nav';
import { getPublicSettings } from '@/api/settings';
import { resolveEnabledFeatures } from '@/libs/enabled-features';

interface Props {
    params: Promise<{ locale: string }>
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale, namespace: 'ToolsHub' });

    return {
        title: t('metaTitle'),
        description: t('metaDescription'),
        alternates: localeAlternates(locale, '/tools'),
        openGraph: {
            type: 'website',
            title: t('metaTitle'),
            description: t('metaDescription'),
            url: `/${locale}/tools`,
        },
    };
}

export default async function ToolsHub() {
    const [t, tHeader, settings] = await Promise.all([
        getTranslations('ToolsHub'),
        getTranslations('Header'),
        getPublicSettings(),
    ]);
    const tools = filterNavByFeatures(TOOLS, resolveEnabledFeatures(settings.enabled_features));

    return (
        <PageShell width="wide" className="flex flex-col gap-8">
            <PageTitle variant="hero" title={t('title')} description={t('intro')} />
            <section className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {tools.map(({ key, href, labelKey, icon }) => (
                    <FeatureCard
                        key={key}
                        href={href}
                        icon={icon}
                        title={tHeader(labelKey)}
                        desc={t(`items.${key}`)}
                    />
                ))}
            </section>
        </PageShell>
    );
}
