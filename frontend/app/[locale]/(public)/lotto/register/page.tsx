import LottoRegisterClient from "@/components/lotto/LottoRegisterClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Lotto');
    return { title: t('registerTitle') };
}

export default async function LottoRegisterPage() {
    const t = await getTranslations('Lotto');

    return (
        <div className="w-full max-w-2xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('registerTitle')}</h1>
            <LottoNav />
            <LottoRegisterClient />
        </div>
    );
}
