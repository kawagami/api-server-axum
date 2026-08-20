import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import VocabClient from "../vocab/vocab-client";
import PageShell from "@/components/page-shell";
import { loadVocabPage } from "../vocab/load";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Vocab");
    return { title: t("titleJa") };
}

export default async function VocabJaPage() {
    const t = await getTranslations("Vocab");
    const { isMember, me, mistakes, leaderboard } = await loadVocabPage("ja");

    return (
        <PageShell width="form" className="flex flex-col gap-6">
            <VocabClient isMember={isMember} initialMe={me} initialMistakes={mistakes}
                initialLeaderboard={leaderboard} language="ja" />
            {/* JMdict 為 CC BY-SA 授權,出處標註是硬需求 */}
            <p className="text-xs text-neutral-400 dark:text-neutral-500 text-center">
                {t("jmdictAttribution")}
            </p>
        </PageShell>
    );
}
