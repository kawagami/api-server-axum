import { getVocabMe } from "@/api/vocab";
import { getTranslations } from "next-intl/server";
import type { Metadata } from "next";
import VocabClient from "./VocabClient";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Vocab");
    return { title: t("title") };
}

export default async function VocabPage() {
    const me = await getVocabMe();

    return (
        <div className="w-full max-w-2xl px-4 py-8">
            <VocabClient initialMe={me} />
        </div>
    );
}
