import { getInvoices } from "@/api/invoices";
import InvoiceListClient from "@/components/invoices/InvoiceListClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

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
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('myInvoicesTitle')}</h1>
            <InvoiceNav />
            <InvoiceListClient initialEntries={entries} />
        </div>
    );
}
