import { getVocabMe, getVocabMistakes } from "@/api/vocab";
import { getTranslations } from "next-intl/server";
import { cookies } from "next/headers";
import type { Metadata } from "next";
import VocabClient from "./VocabClient";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Vocab");
    return { title: t("title") };
}

export default async function VocabPage() {
    // 訪客(無 access_token)也能玩,但不抓會員資料(避免 401 轉登入)
    const isMember = !!(await cookies()).get("access_token")?.value;
    const [me, mistakes] = isMember
        ? await Promise.all([getVocabMe(), getVocabMistakes()])
        : [null, []];

    return (
        <div className="w-full max-w-2xl px-4 py-8">
            <VocabClient initialMe={me} initialMistakes={mistakes} />
        </div>
    );
}
