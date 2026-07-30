import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import { localeAlternates } from '@/libs/seo';
import ProjectIntro from '@/components/home/project-intro';

interface Props {
    params: Promise<{ locale: string }>
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
    const { locale } = await params;
    const t = await getTranslations({ locale, namespace: 'Home' });

    return {
        // 首頁標題自帶站名，用 absolute 跳過 root layout 的 template（否則站名會出現兩次）
        title: { absolute: t('metaTitle') },
        description: t('metaDescription'),
        alternates: localeAlternates(locale, ''),
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
