import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import ProjectIntro from '@/components/home/project-intro';

interface Props {
    params: Promise<{ locale: string }>
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale, namespace: 'Home' });

    return {
        title: t('metaTitle'),
        description: t('metaDescription'),
        alternates: { canonical: `/${locale}` },
        openGraph: {
            type: 'website',
            title: t('metaTitle'),
            description: t('metaDescription'),
            url: `/${locale}`,
        },
    };
}

export default function Home() {
    return <ProjectIntro />;
}
