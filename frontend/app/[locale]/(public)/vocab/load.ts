import { getVocabLeaderboard, getVocabMe, getVocabMistakes } from "@/api/vocab";
import { cookies } from "next/headers";
import type { VocabLanguage, VocabLeaderboard, VocabMe, VocabMistakesPage } from "@/types";
import { MISTAKE_PAGE_SIZE } from "./config";

export interface VocabPageData {
    /** 有 access_token 就是會員 —— 不能用 me !== null 推,me 可能只是這次抓失敗 */
    isMember: boolean;
    me: VocabMe | null;
    mistakes: VocabMistakesPage | null;
    leaderboard: VocabLeaderboard | null;
}

/**
 * /vocab 與 /vocab-ja 的 SSR 資料來源(兩頁只差 language)。
 *
 * 三支端點一次併發,不排隊 —— 之前 leaderboard 單獨 await 會多一趟 RTT。
 * 每支各自 catch:任一支掛掉只讓對應區塊消失,不炸整頁。
 */
export async function loadVocabPage(language: VocabLanguage): Promise<VocabPageData> {
    // 訪客(無 access_token)也能玩,但不抓會員端點(會 401 轉登入)
    const isMember = !!(await cookies()).get("access_token")?.value;
    const [me, mistakes, leaderboard] = await Promise.all([
        isMember ? getVocabMe(language).catch(() => null) : null,
        isMember
            ? getVocabMistakes({ language, limit: MISTAKE_PAGE_SIZE }).catch(() => null)
            : null,
        getVocabLeaderboard(language, "weekly").catch(() => null),
    ]);
    return { isMember, me, mistakes, leaderboard };
}
