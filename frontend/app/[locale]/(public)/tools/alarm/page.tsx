import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import { localeAlternates } from '@/libs/seo';
import Alarm from '@/components/alarm/alarm';

// 標題用 Header 的工具名、描述用 /tools 卡片的說明，維持單一來源
export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
    const { locale } = await params;
    const [tHeader, tHub] = await Promise.all([
        getTranslations({ locale, namespace: 'Header' }),
        getTranslations({ locale, namespace: 'ToolsHub' }),
    ]);

    return {
        title: tHeader('toolAlarm'),
        description: tHub('items.alarm'),
        alternates: localeAlternates(locale, '/tools/alarm'),
    };
}

export default function AlarmPage() {
    return <Alarm />;
}
