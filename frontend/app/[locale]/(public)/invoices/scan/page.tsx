import { getLedgerCategories } from "@/api/ledger";
import InvoiceRegisterClient from "@/components/invoices/InvoiceRegisterClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Invoices');
    return { title: t('registerTitle') };
}

export default async function InvoiceScanPage() {
    const [categories, t] = await Promise.all([
        getLedgerCategories(),
        getTranslations('Invoices'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('registerTitle')} />
            <InvoiceNav />
            <InvoiceRegisterClient categories={categories} />
        </PageShell>
    );
}
