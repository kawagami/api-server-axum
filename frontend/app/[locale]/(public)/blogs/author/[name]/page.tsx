import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import BlogList from '@/components/blogs/blog-list';

interface Props {
    params: Promise<{ locale: string; name: string }>
    searchParams: Promise<{ tag?: string; page?: string }>
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
    const { locale, name } = await params;
    const author = decodeURIComponent(name);
    const t = await getTranslations({ locale, namespace: 'BlogList' });

    const title = t('metaAuthorTitle', { author });

    return {
        title,
        description: t('metaDescription'),
        alternates: { canonical: `/${locale}/blogs/author/${name}` },
        openGraph: {
            type: 'website',
            title,
            description: t('metaDescription'),
            url: `/${locale}/blogs/author/${name}`,
        },
    };
}

export default async function AuthorBlogsPage({ params, searchParams }: Props) {
    const { name } = await params;
    const { tag, page } = await searchParams;
    return (
        <BlogList
            author={decodeURIComponent(name)}
            selectedTag={tag ?? null}
            page={page ? Number(page) : 1}
        />
    );
}
