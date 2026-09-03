"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";
import type { StockDayAll, StockBuybackPeriod, StockChangePaginatedResponse, PaginatedResponse } from "@/types";

export async function patchStockPendingAction(formData: FormData): Promise<void> {
    const id = Number(formData.get('id'));
    await patchOneStockChangePending({ id });
}


export async function patchOneStockChangePending({ id }: { id: string | number }): Promise<void> {
    await adminRequest({
        url: `${process.env.API_URL}/admin/stocks/changes/${id}/pending`,
        method: "PATCH",
    });
    revalidatePath("/");
}

export async function getStockChanges(
    status: string | null = null,
    page = 1,
    perPage = 50,
): Promise<StockChangePaginatedResponse> {
    const url = `${process.env.API_URL}/admin/stocks/changes`;
    const params = new URLSearchParams();
    if (status) params.append("status", status);
    params.append("page", String(page));
    params.append("per_page", String(perPage));

    return adminRequest<StockChangePaginatedResponse>({
        url: `${url}?${params}`,
    });
}

export async function getStockDayAll({ trade_date = "", stock_code = "", page = 1, perPage = 100 } = {}): Promise<PaginatedResponse<StockDayAll>> {
    const url = new URL(`${process.env.API_URL}/admin/stocks/day_all`);
    if (trade_date) url.searchParams.append("trade_date", trade_date);
    if (stock_code) url.searchParams.append("stock_code", stock_code);
    url.searchParams.append("page", String(page));
    url.searchParams.append("per_page", String(perPage));
    const res = await adminRequest<PaginatedResponse<StockDayAll>>({ url: url.toString() });
    return res ?? { data: [], total: 0 };
}

export async function getStockBuybackPeriods(): Promise<StockBuybackPeriod[]> {
    return adminRequest<StockBuybackPeriod[]>({ url: `${process.env.API_URL}/admin/stocks/buyback_periods` });
}

export async function getUnfinishedBuybackPriceGap(): Promise<unknown> {
    return adminRequest({
        url: `${process.env.API_URL}/admin/stocks/buyback_price_gaps`,
        headers: { "Content-Type": "application/json" },
    });
}
