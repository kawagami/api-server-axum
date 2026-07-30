import { getInvoices } from "@/api/invoices";
import InvoiceListClient from "@/components/invoices/InvoiceListClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Invoices');
    return { title: t('winningsTitle') };
}

export default async function InvoiceWinningsPage() {
    const [entries, t] = await Promise.all([
        getInvoices({ won: true, page: 1, per_page: 50 }),
        getTranslations('Invoices'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('winningsTitle')} />
            <InvoiceNav />
            <InvoiceListClient initialEntries={entries} lockWon />
        </PageShell>
    );
}
