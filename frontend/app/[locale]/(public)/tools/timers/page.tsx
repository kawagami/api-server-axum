import type { Metadata } from 'next';
import { getTranslations } from 'next-intl/server';
import { localeAlternates } from '@/libs/seo';
import PageShell from '@/components/page-shell';
import PageTitle from '@/components/page-title';
import HourlyChimeCard from './hourly-chime-card';
import CountdownCard from './countdown-card';
import AlarmCard from './alarm-card';

// 標題用 Header 的工具名、描述用 /tools 卡片的說明，維持單一來源
export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
    const { locale } = await params;
    const [tHeader, tHub] = await Promise.all([
        getTranslations({ locale, namespace: 'Header' }),
        getTranslations({ locale, namespace: 'ToolsHub' }),
    ]);

    return {
        title: tHeader('toolTimers'),
        description: tHub('items.timers'),
        alternates: localeAlternates(locale, '/tools/timers'),
    };
}

// 三張卡片同時掛載（不是分頁切換）：切 tab 會卸載元件、計時就死了，
// 並存反而解鎖「一邊倒數一邊開整點報時」這種原本做不到的組合。
export default async function TimersPage() {
    const t = await getTranslations('Timers');

    return (
        <PageShell width="wide" className="flex flex-col gap-6">
            <PageTitle title={t('title')} description={t('intro')} />
            <HourlyChimeCard />
            <div className="grid gap-6 md:grid-cols-2">
                <CountdownCard />
                <AlarmCard />
            </div>
        </PageShell>
    );
}
