import { getInvoiceDraws } from "@/api/invoices";
import InvoiceDrawsClient from "@/components/invoices/InvoiceDrawsClient";
import InvoiceNav from "@/components/invoices/InvoiceNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

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
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('drawsTitle')}</h1>
            <InvoiceNav />
            <InvoiceDrawsClient initialDraws={draws} />
        </div>
    );
}
