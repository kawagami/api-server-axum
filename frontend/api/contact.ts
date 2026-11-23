"use server";

import { fetchApi } from "@/libs/fetchApi";
import { clientIpHeaders } from "@/libs/client-ip";
import adminRequest from "@/libs/adminRequest";
import type { ContactMessage, NewContactMessage, PaginatedResponse } from "@/types";

// 公開端:訪客留言(不需登入)
export async function postContactMessage(input: NewContactMessage): Promise<ContactMessage> {
    return fetchApi(`${process.env.API_URL}/messages`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...(await clientIpHeaders()) },
        body: JSON.stringify(input),
        cache: "no-store",
    });
}

// 後台:留言分頁列表(需 message:read)。回整包不解包 —— total 要不要用由呼叫端決定
export async function getContactMessages(
    page = 1,
    per_page = 50,
): Promise<PaginatedResponse<ContactMessage>> {
    const res = await adminRequest<PaginatedResponse<ContactMessage>>({
        url: `${process.env.API_URL}/admin/messages?page=${page}&per_page=${per_page}`,
    });
    return res ?? { data: [], total: 0 };
}

// 後台:刪除留言(需 message:delete;單筆也走這支,傳一個 id)。ids 放 query 讓後端稽核記得到刪了哪幾筆;
// 單次上限 MAX_BATCH_IDS(components/admin/bulk-select.tsx)。回實際刪掉的 id(不存在的略過)
export async function deleteContactMessages(ids: number[]): Promise<number[]> {
    // Server Action 的參數是 client 送來的、不可信：只放行正整數，不讓 `1&x=y` 這種值拼進 URL
    const safe = ids.filter(id => Number.isSafeInteger(id) && id > 0);
    if (safe.length === 0) return [];
    const res = await adminRequest<{ deleted: number[] }>({
        url: `${process.env.API_URL}/admin/messages?ids=${safe.join(",")}`,
        method: "DELETE",
    });
    return res?.deleted ?? [];
}
