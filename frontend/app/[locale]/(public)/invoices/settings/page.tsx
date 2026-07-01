import { getCurrentMember } from "@/api/members";
import NotifySettingsClient from "@/components/invoices/NotifySettingsClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Invoices');
    return { title: t('settingsTitle') };
}

export default async function InvoiceSettingsPage() {
    const [member, t] = await Promise.all([
        getCurrentMember(),
        getTranslations('Invoices'),
    ]);

    return (
        <div className="w-full max-w-2xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('settingsTitle')}</h1>
            <InvoiceNav />
            <NotifySettingsClient hasEmail={!!member.email} email={member.email} initialEnabled={member.lottery_notify_enabled} />
        </div>
    );
}
