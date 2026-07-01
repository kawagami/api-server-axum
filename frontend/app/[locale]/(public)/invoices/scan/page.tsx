import { getLedgerCategories } from "@/api/ledger";
import InvoiceRegisterClient from "@/components/invoices/InvoiceRegisterClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

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
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('registerTitle')}</h1>
            <InvoiceNav />
            <InvoiceRegisterClient categories={categories} />
        </div>
    );
}
