"use server";

import memberRequest from "@/libs/memberRequest";
import type { VocabAnswer, VocabAnswerInput, VocabLanguage, VocabLeaderboard, VocabLeaderboardPeriod, VocabMe, VocabMistakeSort, VocabMistakesPage, VocabRunMode, VocabStartRun } from "@/types";

export async function getVocabMe(language: VocabLanguage = 'en'): Promise<VocabMe> {
    return memberRequest<VocabMe>({
        url: `${process.env.API_URL}/member/vocab/me?language=${language}`,
    });
}

export interface VocabMistakeQuery {
    language?: VocabLanguage;
    /** 表記 / 讀音 / 釋義模糊搜尋 */
    q?: string;
    sort?: VocabMistakeSort;
    /** 只看未掌握(答錯 > 答對) */
    unmastered?: boolean;
    /** 後端上限 100 */
    limit?: number;
    offset?: number;
}

/** 錯題本一頁;後端強制分頁(上限 100),不要期待一次拿完 */
export async function getVocabMistakes(query: VocabMistakeQuery = {}): Promise<VocabMistakesPage> {
    const params = new URLSearchParams({ language: query.language ?? 'en' });
    if (query.q?.trim()) params.set('q', query.q.trim());
    if (query.sort) params.set('sort', query.sort);
    if (query.unmastered) params.set('unmastered', 'true');
    if (query.limit != null) params.set('limit', String(query.limit));
    if (query.offset) params.set('offset', String(query.offset));
    return memberRequest<VocabMistakesPage>({
        url: `${process.env.API_URL}/member/vocab/mistakes?${params}`,
    });
}

// 訪客也能看榜(端點選擇性驗證);登入時回應多帶自己的名次
export async function getVocabLeaderboard(
    language: VocabLanguage = 'en',
    period: VocabLeaderboardPeriod = 'weekly',
): Promise<VocabLeaderboard> {
    return memberRequest<VocabLeaderboard>({
        url: `${process.env.API_URL}/member/vocab/leaderboard?language=${language}&period=${period}`,
    });
}

export async function startVocabRun(
    mode: VocabRunMode = 'survival',
    durationMinutes?: number,
    language: VocabLanguage = 'en',
): Promise<VocabStartRun> {
    return memberRequest<VocabStartRun>({
        url: `${process.env.API_URL}/member/vocab/runs`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode, duration_minutes: durationMinutes, language }),
    });
}

export async function finishVocabRun(runId: string): Promise<VocabAnswer> {
    return memberRequest<VocabAnswer>({
        url: `${process.env.API_URL}/member/vocab/runs/${runId}/finish`,
        method: 'POST',
    });
}

export async function answerVocabRun(runId: string, input: VocabAnswerInput): Promise<VocabAnswer> {
    return memberRequest<VocabAnswer>({
        url: `${process.env.API_URL}/member/vocab/runs/${runId}/answer`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}
