import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import VocabClient from "./vocab-client";
import PageShell from "@/components/page-shell";
import { loadVocabPage } from "./load";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Vocab");
    return { title: t("title") };
}

export default async function VocabPage() {
    const { isMember, me, mistakes, leaderboard } = await loadVocabPage("en");

    return (
        <PageShell width="form">
            <VocabClient isMember={isMember} initialMe={me} initialMistakes={mistakes}
                initialLeaderboard={leaderboard} />
        </PageShell>
    );
}
