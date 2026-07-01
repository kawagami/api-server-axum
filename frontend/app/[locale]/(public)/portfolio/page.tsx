import { getPortfolioSummary } from "@/api/portfolio";
import PortfolioClient from "@/components/portfolio/PortfolioClient";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations('Portfolio');
    return { title: t('title') };
}

export default async function PortfolioPage() {
    const [entries, t] = await Promise.all([
        getPortfolioSummary(),
        getTranslations('Portfolio'),
    ]);

    return (
        <div className="w-full max-w-4xl px-4 py-8">
            <h1 className="text-2xl font-bold mb-6">{t('title')}</h1>
            <PortfolioClient initialEntries={entries} />
        </div>
    );
}
