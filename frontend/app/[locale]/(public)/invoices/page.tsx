import { getInvoices } from "@/api/invoices";
import InvoiceListClient from "@/components/invoices/invoice-list-client";
import InvoiceNav from "@/components/invoices/invoice-nav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Invoices');
    return { title: t('myInvoicesTitle') };
}

export default async function InvoicesPage() {
    const [entries, t] = await Promise.all([
        getInvoices({ page: 1, per_page: 50 }),
        getTranslations('Invoices'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('myInvoicesTitle')} />
            <InvoiceNav />
            <InvoiceListClient initialEntries={entries} />
        </PageShell>
    );
}
