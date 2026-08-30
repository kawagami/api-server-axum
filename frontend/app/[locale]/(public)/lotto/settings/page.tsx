import { getCurrentMember } from "@/api/members";
import { setLottoNotify } from "@/api/lotto";
import NotifyToggle from "@/components/notify-toggle";
import LottoNav from "@/components/lotto/lotto-nav";
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
            <NotifyToggle
                namespace="Lotto"
                action={setLottoNotify}
                hasEmail={!!member.email}
                email={member.email}
                initialEnabled={member.lotto_notify_enabled}
            />
        </PageShell>
    );
}
