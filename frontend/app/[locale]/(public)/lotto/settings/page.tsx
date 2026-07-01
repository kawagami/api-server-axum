import { getCurrentMember } from "@/api/members";
import LottoNotifySettingsClient from "@/components/lotto/LottoNotifySettingsClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

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
        <div className="w-full max-w-2xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('settingsTitle')}</h1>
            <LottoNav />
            <LottoNotifySettingsClient hasEmail={!!member.email} email={member.email} initialEnabled={member.lotto_notify_enabled} />
        </div>
    );
}
