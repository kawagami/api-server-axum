import { getLottoTickets } from "@/api/lotto";
import LottoListClient from "@/components/lotto/lotto-list-client";
import LottoNav from "@/components/lotto/lotto-nav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Lotto');
    return { title: t('myTicketsTitle') };
}

export default async function LottoPage() {
    const [entries, t] = await Promise.all([
        getLottoTickets({ page: 1, per_page: 50 }),
        getTranslations('Lotto'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('myTicketsTitle')} />
            <LottoNav />
            <LottoListClient initialEntries={entries} />
        </PageShell>
    );
}
