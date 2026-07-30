import LottoRegisterClient from "@/components/lotto/LottoRegisterClient";
import LottoNav from "@/components/lotto/LottoNav";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import PageShell from "@/components/page-shell";
import PageTitle from "@/components/page-title";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Lotto');
    return { title: t('registerTitle') };
}

export default async function LottoRegisterPage() {
    const t = await getTranslations('Lotto');

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <PageTitle title={t('registerTitle')} />
            <LottoNav />
            <LottoRegisterClient />
        </PageShell>
    );
}
