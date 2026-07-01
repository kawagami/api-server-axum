import { getLedgerCategories, getLedger, getLedgerSummary } from "@/api/ledger";
import LedgerClient from "@/components/ledger/LedgerClient";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Ledger');
    return { title: t('title') };
}

export default async function LedgerPage() {
    const [categories, entries, summary, t] = await Promise.all([
        getLedgerCategories(),
        getLedger({ page: 1, per_page: 50 }),
        getLedgerSummary(),
        getTranslations('Ledger'),
    ]);

    return (
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('title')}</h1>
            <LedgerClient categories={categories} initialEntries={entries} initialSummary={summary} />
        </div>
    );
}
