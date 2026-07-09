"use server";

import memberRequest from "@/libs/memberRequest";
import type { VocabAnswer, VocabAnswerInput, VocabMe, VocabMistake, VocabRunMode, VocabStartRun } from "@/types";

export async function getVocabMe(): Promise<VocabMe> {
    return memberRequest<VocabMe>({
        url: `${process.env.API_URL}/member/vocab/me`,
    });
}

export async function getVocabMistakes(): Promise<VocabMistake[]> {
    return memberRequest<VocabMistake[]>({
        url: `${process.env.API_URL}/member/vocab/mistakes`,
    });
}

export async function startVocabRun(mode: VocabRunMode = 'survival'): Promise<VocabStartRun> {
    return memberRequest<VocabStartRun>({
        url: `${process.env.API_URL}/member/vocab/runs`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode }),
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
