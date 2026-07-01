import { getLottoTickets } from "@/api/lotto";
import LottoListClient from "@/components/lotto/LottoListClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

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
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('myTicketsTitle')}</h1>
            <LottoNav />
            <LottoListClient initialEntries={entries} />
        </div>
    );
}
