import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import FeatureCard from '@/components/feature-card';
import { GAMES } from '@/libs/site-nav';

interface Props {
    params: Promise<{ locale: string }>
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale, namespace: 'GamesHub' });

    return {
        title: t('metaTitle'),
        description: t('metaDescription'),
        alternates: { canonical: `/${locale}/games` },
        openGraph: {
            type: 'website',
            title: t('metaTitle'),
            description: t('metaDescription'),
            url: `/${locale}/games`,
        },
    };
}

export default async function GamesHub() {
    const t = await getTranslations('GamesHub');
    const tHeader = await getTranslations('Header');

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto">
            <div className="max-w-5xl mx-auto px-4 pb-12">
                <section className="text-center pt-4 pb-8">
                    <h1 className="text-2xl sm:text-3xl font-bold text-neutral-800 dark:text-neutral-100 mb-3">
                        {t('title')}
                    </h1>
                    <p className="max-w-2xl mx-auto text-neutral-600 dark:text-neutral-300 text-sm sm:text-base">
                        {t('intro')}
                    </p>
                </section>
                <section className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                    {GAMES.map(({ key, href, labelKey, icon }) => (
                        <FeatureCard
                            key={key}
                            href={href}
                            icon={icon}
                            title={tHeader(labelKey)}
                            desc={t(`items.${key}`)}
                        />
                    ))}
                </section>
            </div>
        </div>
    );
}
