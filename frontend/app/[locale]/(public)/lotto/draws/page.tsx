import { getLottoDraws } from "@/api/lotto";
import LottoDrawsClient from "@/components/lotto/LottoDrawsClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Lotto');
    return { title: t('drawsTitle') };
}

export default async function LottoDrawsPage() {
    const [draws, t] = await Promise.all([
        getLottoDraws({ limit: 20 }),
        getTranslations('Lotto'),
    ]);

    return (
        <PageShell className="flex flex-col gap-6">
            <PageTitle title={t('drawsTitle')} />
            <LottoNav />
            <LottoDrawsClient initialDraws={draws} />
        </PageShell>
    );
}
