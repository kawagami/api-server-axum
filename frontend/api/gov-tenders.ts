"use server";

import adminRequest from "@/libs/adminRequest";
import type { GovTender } from "@/types";

export interface GetGovTendersParams {
    q?: string;
    keyword?: string;
    tender_type?: string;
    page?: number;
    per_page?: number;
}

interface GovTenderListResponse {
    data: GovTender[];
    total: number;
}

export async function getGovTenders({
    q,
    keyword,
    tender_type,
    page = 1,
    per_page = 50,
}: GetGovTendersParams = {}): Promise<GovTender[]> {
    const params = new URLSearchParams();
    if (q) params.set('q', q);
    if (keyword) params.set('keyword', keyword);
    if (tender_type) params.set('tender_type', tender_type);
    params.set('page', String(page));
    params.set('per_page', String(per_page));

    const res = await adminRequest<GovTenderListResponse>({
        url: `${process.env.API_URL}/admin/gov_tenders?${params}`,
    });
    return res?.data ?? [];
}
