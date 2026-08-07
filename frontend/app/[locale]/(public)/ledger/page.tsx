import { getLedgerCategories, getLedger, getLedgerSummary } from "@/api/ledger";
import LedgerClient from "./ledger-client";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import { Link } from "@/i18n/navigation";
import { LayoutDashboard } from "lucide-react";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Ledger');
    return { title: t('title') };
}

export default async function LedgerPage() {
    const [categories, entries, summary, t, tHeader] = await Promise.all([
        getLedgerCategories(),
        getLedger({ page: 1, per_page: 50 }),
        getLedgerSummary(),
        getTranslations('Ledger'),
        getTranslations('Header'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle
                title={t('title')}
                actions={
                    <Link
                        href="/dashboard"
                        className="flex items-center gap-1 text-sm text-neutral-500 dark:text-neutral-400 hover:text-primary-500 dark:hover:text-primary-400 transition-colors"
                    >
                        <LayoutDashboard size={16} />
                        {tHeader('dashboard')}
                    </Link>
                }
            />
            <LedgerClient categories={categories} initialPage={entries} initialSummary={summary} />
        </PageShell>
    );
}
