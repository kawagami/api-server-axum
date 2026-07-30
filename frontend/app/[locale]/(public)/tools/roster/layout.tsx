import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import { localeAlternates } from '@/libs/seo';

// page.tsx 是 client component，不能自己 export metadata，所以放這裡
export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
    const { locale } = await params;
    const [tHeader, tHub] = await Promise.all([
        getTranslations({ locale, namespace: 'Header' }),
        getTranslations({ locale, namespace: 'ToolsHub' }),
    ]);

    return {
        title: tHeader('toolRoster'),
        description: tHub('items.roster'),
        alternates: localeAlternates(locale, '/tools/roster'),
    };
}

export default function Layout({ children }: { children: React.ReactNode }) {
    return children;
}
