"use server";

import memberRequest from "@/libs/memberRequest";
import type {
    LottoTicket,
    LottoInput,
    LottoListParams,
    LottoDraw,
    LottoDrawParams,
} from "@/types";

export async function getLottoTickets(params: LottoListParams = {}): Promise<LottoTicket[]> {
    const qs = new URLSearchParams();
    if (params.game) qs.set('game', params.game);
    if (params.status) qs.set('status', params.status);
    if (params.page) qs.set('page', String(params.page));
    if (params.per_page) qs.set('per_page', String(params.per_page));
    const q = qs.toString();
    return memberRequest<LottoTicket[]>({
        url: `${process.env.API_URL}/member/lotto${q ? `?${q}` : ''}`,
    });
}

export async function getLottoTicket(id: string): Promise<LottoTicket> {
    return memberRequest<LottoTicket>({
        url: `${process.env.API_URL}/member/lotto/${id}`,
    });
}

// 批次登錄；回傳本批建立的所有注。422 = 號碼/範圍/重複/缺漏不合法
export async function postLotto(input: LottoInput): Promise<LottoTicket[]> {
    return memberRequest<LottoTicket[]>({
        url: `${process.env.API_URL}/member/lotto`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input),
    });
}

export async function deleteLottoTicket(id: string): Promise<null> {
    return memberRequest<null>({
        url: `${process.env.API_URL}/member/lotto/${id}`,
        method: 'DELETE',
    });
}

// 近期開獎號碼（需登入）
export async function getLottoDraws(params: LottoDrawParams = {}): Promise<LottoDraw[]> {
    const qs = new URLSearchParams();
    if (params.game) qs.set('game', params.game);
    if (params.limit) qs.set('limit', String(params.limit));
    const q = qs.toString();
    return memberRequest<LottoDraw[]>({
        url: `${process.env.API_URL}/member/lotto/draws${q ? `?${q}` : ''}`,
    });
}

// 開關中獎 email 通知；422 = 此帳號未綁定 email
export async function setLottoNotify(enabled: boolean): Promise<{ enabled: boolean }> {
    return memberRequest<{ enabled: boolean }>({
        url: `${process.env.API_URL}/member/lotto/notify`,
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled }),
    });
}
