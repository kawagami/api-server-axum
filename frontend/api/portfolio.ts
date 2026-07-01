"use server";

import memberRequest from "@/libs/memberRequest";
import type { PortfolioEntry, PortfolioEntryInput, PortfolioSummaryEntry, HistoryRecord } from "@/types";

export async function getPortfolioSummary(): Promise<PortfolioSummaryEntry[]> {
    return memberRequest<PortfolioSummaryEntry[]>({
        url: `${process.env.API_URL}/member/portfolio/summary`,
    });
}

export async function getPortfolioHistory(portfolioId: string): Promise<HistoryRecord[]> {
    return memberRequest<HistoryRecord[]>({
        url: `${process.env.API_URL}/member/portfolio/${portfolioId}/history`,
    });
}

export async function postPortfolio(input: PortfolioEntryInput): Promise<PortfolioEntry> {
    return memberRequest<PortfolioEntry>({
        url: `${process.env.API_URL}/member/portfolio`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}

export async function putPortfolio(id: string, input: PortfolioEntryInput): Promise<PortfolioEntry> {
    return memberRequest<PortfolioEntry>({
        url: `${process.env.API_URL}/member/portfolio/${id}`,
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}

export async function deletePortfolio(id: string): Promise<null> {
    return memberRequest<null>({
        url: `${process.env.API_URL}/member/portfolio/${id}`,
        method: 'DELETE',
    });
}
