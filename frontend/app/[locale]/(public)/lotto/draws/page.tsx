import { getLottoDraws } from "@/api/lotto";
import LottoDrawsClient from "@/components/lotto/LottoDrawsClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

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
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('drawsTitle')}</h1>
            <LottoNav />
            <LottoDrawsClient initialDraws={draws} />
        </div>
    );
}
