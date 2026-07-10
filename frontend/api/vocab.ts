"use server";

import memberRequest from "@/libs/memberRequest";
import type { VocabAnswer, VocabAnswerInput, VocabLanguage, VocabLeaderboard, VocabLeaderboardPeriod, VocabMe, VocabMistake, VocabRunMode, VocabStartRun } from "@/types";

export async function getVocabMe(language: VocabLanguage = 'en'): Promise<VocabMe> {
    return memberRequest<VocabMe>({
        url: `${process.env.API_URL}/member/vocab/me?language=${language}`,
    });
}

export async function getVocabMistakes(language: VocabLanguage = 'en'): Promise<VocabMistake[]> {
    return memberRequest<VocabMistake[]>({
        url: `${process.env.API_URL}/member/vocab/mistakes?language=${language}`,
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
