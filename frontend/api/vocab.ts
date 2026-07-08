"use server";

import memberRequest from "@/libs/memberRequest";
import type { VocabAnswer, VocabAnswerInput, VocabMe, VocabStartRun } from "@/types";

export async function getVocabMe(): Promise<VocabMe> {
    return memberRequest<VocabMe>({
        url: `${process.env.API_URL}/member/vocab/me`,
    });
}

export async function startVocabRun(): Promise<VocabStartRun> {
    return memberRequest<VocabStartRun>({
        url: `${process.env.API_URL}/member/vocab/runs`,
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
