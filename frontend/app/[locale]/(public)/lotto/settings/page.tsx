import { getCurrentMember } from "@/api/members";
import LottoNotifySettingsClient from "@/components/lotto/LottoNotifySettingsClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Lotto');
    return { title: t('settingsTitle') };
}

export default async function LottoSettingsPage() {
    const [member, t] = await Promise.all([
        getCurrentMember(),
        getTranslations('Lotto'),
    ]);

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t('settingsTitle')} />
            <LottoNav />
            <LottoNotifySettingsClient hasEmail={!!member.email} email={member.email} initialEnabled={member.lotto_notify_enabled} />
        </PageShell>
    );
}
