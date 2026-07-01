"use server";

import memberRequest from "@/libs/memberRequest";
import type { LedgerEntry, LedgerInput, LedgerCategories, LedgerSummary } from "@/types";

export interface LedgerListParams {
    kind?: string;
    category?: string;
    from?: string;
    to?: string;
    page?: number;
    per_page?: number;
}

export async function getLedgerCategories(): Promise<LedgerCategories> {
    return memberRequest<LedgerCategories>({
        url: `${process.env.API_URL}/member/ledger/categories`,
    });
}

export async function getLedger(params: LedgerListParams = {}): Promise<LedgerEntry[]> {
    const qs = new URLSearchParams();
    if (params.kind) qs.set('kind', params.kind);
    if (params.category) qs.set('category', params.category);
    if (params.from) qs.set('from', params.from);
    if (params.to) qs.set('to', params.to);
    if (params.page) qs.set('page', String(params.page));
    if (params.per_page) qs.set('per_page', String(params.per_page));
    const q = qs.toString();
    return memberRequest<LedgerEntry[]>({
        url: `${process.env.API_URL}/member/ledger${q ? `?${q}` : ''}`,
    });
}

export async function getLedgerSummary(params: { from?: string; to?: string } = {}): Promise<LedgerSummary> {
    const qs = new URLSearchParams();
    if (params.from) qs.set('from', params.from);
    if (params.to) qs.set('to', params.to);
    const q = qs.toString();
    return memberRequest<LedgerSummary>({
        url: `${process.env.API_URL}/member/ledger/summary${q ? `?${q}` : ''}`,
    });
}

export async function postLedger(input: LedgerInput): Promise<LedgerEntry> {
    return memberRequest<LedgerEntry>({
        url: `${process.env.API_URL}/member/ledger`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}

export async function putLedger(id: string, input: LedgerInput): Promise<LedgerEntry> {
    return memberRequest<LedgerEntry>({
        url: `${process.env.API_URL}/member/ledger/${id}`,
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}

export async function deleteLedger(id: string): Promise<null> {
    return memberRequest<null>({
        url: `${process.env.API_URL}/member/ledger/${id}`,
        method: 'DELETE',
    });
}
