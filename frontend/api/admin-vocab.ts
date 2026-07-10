"use server";

import adminRequest from "@/libs/adminRequest";
import type { AdminVocabWord, UpdateVocabWordInput } from "@/types";

export interface GetAdminVocabWordsParams {
    language?: string;
    difficulty?: number;
    enabled?: boolean;
    q?: string;
    sort?: string; // 'wrong' = 錯最多優先
    page?: number;
    per_page?: number;
}

export interface AdminVocabWordsResponse {
    data: AdminVocabWord[];
    total: number;
}

export async function getAdminVocabWords({
    language,
    difficulty,
    enabled,
    q,
    sort,
    page = 1,
    per_page = 50,
}: GetAdminVocabWordsParams = {}): Promise<AdminVocabWordsResponse> {
    const params = new URLSearchParams();
    if (language) params.set('language', language);
    if (difficulty != null) params.set('difficulty', String(difficulty));
    if (enabled != null) params.set('enabled', String(enabled));
    if (q) params.set('q', q);
    if (sort) params.set('sort', sort);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    const res = await adminRequest<AdminVocabWordsResponse>({
        url: `${process.env.API_URL}/admin/vocab/words?${params}`,
    });
    return res ?? { data: [], total: 0 };
}

export async function updateAdminVocabWord(id: number, input: UpdateVocabWordInput): Promise<void> {
    await adminRequest<null>({
        url: `${process.env.API_URL}/admin/vocab/words/${id}`,
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}
