import { getInvoiceDraws } from "@/api/invoices";
import InvoiceDrawsClient from "@/components/invoices/InvoiceDrawsClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Invoices');
    return { title: t('drawsTitle') };
}

export default async function InvoiceDrawsPage() {
    const [draws, t] = await Promise.all([
        getInvoiceDraws({ limit: 12 }),
        getTranslations('Invoices'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('drawsTitle')} />
            <InvoiceNav />
            <InvoiceDrawsClient initialDraws={draws} />
        </PageShell>
    );
}
