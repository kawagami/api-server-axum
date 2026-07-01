import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import BlogList from '@/components/blogs/blog-list';

interface Props {
    params: Promise<{ locale: string }>
    searchParams: Promise<{ tag?: string; page?: string }>
}

export async function generateMetadata({ params, searchParams }: Props): Promise<Metadata> {
    const [{ locale }, { tag }] = await Promise.all([params, searchParams]);
    const t = await getTranslations({ locale, namespace: 'BlogList' });

    const title = tag ? t('metaTaggedTitle', { tag }) : t('metaTitle');

    return {
        title,
        description: t('metaDescription'),
        alternates: { canonical: `/${locale}/blogs` },
        openGraph: {
            type: 'website',
            title,
            description: t('metaDescription'),
            url: `/${locale}/blogs`,
        },
    };
}

export default async function BlogsPage({ searchParams }: Props) {
    const { tag, page } = await searchParams;
    return <BlogList selectedTag={tag ?? null} page={page ? Number(page) : 1} />;
}
