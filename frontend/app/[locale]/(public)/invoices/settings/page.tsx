import { getCurrentMember } from "@/api/members";
import NotifySettingsClient from "@/components/invoices/notify-settings-client";
import InvoiceNav from "@/components/invoices/invoice-nav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

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
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t('settingsTitle')} />
            <InvoiceNav />
            <NotifySettingsClient hasEmail={!!member.email} email={member.email} initialEnabled={member.lottery_notify_enabled} />
        </PageShell>
    );
}
