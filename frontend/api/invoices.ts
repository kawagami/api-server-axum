"use server";

import memberRequest from "@/libs/memberRequest";
import type { Invoice, InvoiceInput, InvoiceListParams } from "@/types";

export async function getInvoices(params: InvoiceListParams = {}): Promise<Invoice[]> {
    const qs = new URLSearchParams();
    if (params.period) qs.set('period', params.period);
    if (params.won !== undefined) qs.set('won', String(params.won));
    if (params.page) qs.set('page', String(params.page));
    if (params.per_page) qs.set('per_page', String(params.per_page));
    const q = qs.toString();
    return memberRequest<Invoice[]>({
        url: `${process.env.API_URL}/member/invoices${q ? `?${q}` : ''}`,
    });
}

export async function getInvoice(id: string): Promise<Invoice> {
    return memberRequest<Invoice>({
        url: `${process.env.API_URL}/member/invoices/${id}`,
    });
}

export async function postInvoice(input: InvoiceInput): Promise<Invoice> {
    return memberRequest<Invoice>({
        url: `${process.env.API_URL}/member/invoices`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}

export async function deleteInvoice(id: string): Promise<null> {
    return memberRequest<null>({
        url: `${process.env.API_URL}/member/invoices/${id}`,
        method: 'DELETE',
    });
}

// 開關中獎 email 通知；422 = 此帳號未綁定 email
export async function setInvoiceNotify(enabled: boolean): Promise<{ enabled: boolean }> {
    return memberRequest<{ enabled: boolean }>({
        url: `${process.env.API_URL}/member/invoices/notify`,
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled }),
    });
}
