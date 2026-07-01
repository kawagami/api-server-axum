"use server";

import { revalidatePath } from "next/cache";
import adminRequest from "@/libs/adminRequest";
import type { StockDayAll, StockBuybackPeriod, StockChangePaginatedResponse } from "@/types";

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

function isValidDateFormat(dateStr: string): boolean {
    if (!/^\d{8}$/.test(dateStr)) return false;
    const month = parseInt(dateStr.substring(4, 6), 10);
    const day = parseInt(dateStr.substring(6, 8), 10);
    if (month < 1 || month > 12) return false;
    const daysInMonth = new Date(parseInt(dateStr.substring(0, 4), 10), month, 0).getDate();
    return day >= 1 && day <= daysInMonth;
}

export async function fetchStockClosingPricePair({ stock_no, start_date, end_date }: { stock_no: string; start_date: string; end_date: string }): Promise<unknown> {
    if (!isValidDateFormat(start_date) || !isValidDateFormat(end_date)) throw new Error("日期格式錯誤，必須是 YYYYMMDD 格式");
    const url = new URL(`${process.env.API_URL}/admin/stocks/closing_price_stats`);
    url.searchParams.append("stock_no", stock_no);
    url.searchParams.append("start_date", start_date);
    url.searchParams.append("end_date", end_date);
    return adminRequest({ url: url.toString(), method: "GET" });
}

export async function getStockDayAll({ trade_date = "", stock_code = "", page = 1, perPage = 100 } = {}): Promise<StockDayAll[]> {
    const url = new URL(`${process.env.API_URL}/admin/stocks/day_all`);
    if (trade_date) url.searchParams.append("trade_date", trade_date);
    if (stock_code) url.searchParams.append("stock_code", stock_code);
    url.searchParams.append("page", String(page));
    url.searchParams.append("per_page", String(perPage));
    return adminRequest<StockDayAll[]>({ url: url.toString() });
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
