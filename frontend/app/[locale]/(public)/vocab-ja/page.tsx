import { getVocabLeaderboard, getVocabMe, getVocabMistakes } from "@/api/vocab";
import { getTranslations } from "next-intl/server";
import { cookies } from "next/headers";
import type { Metadata } from "next";
import VocabClient from "../vocab/VocabClient";

export async function generateMetadata(): Promise<Metadata> {
    const t = await getTranslations("Vocab");
    return { title: t("titleJa") };
}

export default async function VocabJaPage() {
    const t = await getTranslations("Vocab");
    // 訪客(無 access_token)也能玩,但不抓會員資料(避免 401 轉登入)
    const isMember = !!(await cookies()).get("access_token")?.value;
    const [me, mistakes] = isMember
        ? await Promise.all([getVocabMe("ja"), getVocabMistakes("ja")])
        : [null, []];
    const leaderboard = await getVocabLeaderboard("ja", "weekly").catch(() => null);

    return (
        <div className="w-full max-w-2xl px-4 py-8 flex flex-col gap-6">
            <VocabClient initialMe={me} initialMistakes={mistakes} language="ja" initialLeaderboard={leaderboard} />
            {/* JMdict 為 CC BY-SA 授權,出處標註是硬需求 */}
            <p className="text-xs text-neutral-400 dark:text-neutral-500 text-center">
                {t("jmdictAttribution")}
            </p>
        </div>
    );
}
