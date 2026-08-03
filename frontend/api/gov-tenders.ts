"use server";

import adminRequest from "@/libs/adminRequest";
import type { GovTender, PaginatedResponse } from "@/types";

export interface GetGovTendersParams {
    q?: string;
    keyword?: string;
    tender_type?: string;
    page?: number;
    per_page?: number;
}

export async function getGovTenders({
    q,
    keyword,
    tender_type,
    page = 1,
    per_page = 50,
}: GetGovTendersParams = {}): Promise<PaginatedResponse<GovTender>> {
    const params = new URLSearchParams();
    if (q) params.set('q', q);
    if (keyword) params.set('keyword', keyword);
    if (tender_type) params.set('tender_type', tender_type);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    const res = await adminRequest<PaginatedResponse<GovTender>>({
        url: `${process.env.API_URL}/admin/gov_tenders?${params}`,
    });
    return res ?? { data: [], total: 0 };
}

export async function getGovTenderTypes(): Promise<string[]> {
    const res = await adminRequest<string[]>({
        url: `${process.env.API_URL}/admin/gov_tenders/types`,
    });
    return res ?? [];
}
