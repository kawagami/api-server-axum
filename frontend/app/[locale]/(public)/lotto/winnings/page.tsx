import { getLottoTickets } from "@/api/lotto";
import LottoListClient from "@/components/lotto/LottoListClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Lotto');
    return { title: t('winningsTitle') };
}

export default async function LottoWinningsPage() {
    const [entries, t] = await Promise.all([
        getLottoTickets({ status: 'won', page: 1, per_page: 50 }),
        getTranslations('Lotto'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('winningsTitle')} />
            <LottoNav />
            <LottoListClient initialEntries={entries} lockWon />
        </PageShell>
    );
}
