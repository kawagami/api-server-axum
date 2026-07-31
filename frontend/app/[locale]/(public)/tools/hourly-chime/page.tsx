import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import { localeAlternates } from '@/libs/seo';
import HourlyChime from './hourly-chime';

// 標題用 Header 的工具名、描述用 /tools 卡片的說明，維持單一來源
export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
    const { locale } = await params;
    const [tHeader, tHub] = await Promise.all([
        getTranslations({ locale, namespace: 'Header' }),
        getTranslations({ locale, namespace: 'ToolsHub' }),
    ]);

    return {
        title: tHeader('toolHourlyChime'),
        description: tHub('items.hourlyChime'),
        alternates: localeAlternates(locale, '/tools/hourly-chime'),
    };
}

export default function HourlyChimePage() {
    return <HourlyChime />;
}
