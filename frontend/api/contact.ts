"use server";

import { fetchApi } from "@/libs/fetchApi";
import adminRequest from "@/libs/adminRequest";
import type { ContactMessage, NewContactMessage } from "@/types";

// 公開端:訪客留言(不需登入)
export async function postContactMessage(input: NewContactMessage): Promise<ContactMessage> {
    return fetchApi(`${process.env.API_URL}/messages`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(input),
        cache: "no-store",
    });
}

interface ContactMessageListResponse {
    data: ContactMessage[];
    total: number;
}

// 後台:留言分頁列表(需 message:read)
export async function getContactMessages(page = 1, per_page = 50): Promise<ContactMessage[]> {
    const res = await adminRequest<ContactMessageListResponse>({
        url: `${process.env.API_URL}/admin/messages?page=${page}&per_page=${per_page}`,
    });
    return res?.data ?? [];
}

// 後台:刪除留言(需 message:delete)
export async function deleteContactMessage(id: number): Promise<void> {
    await adminRequest<null>({
        url: `${process.env.API_URL}/admin/messages/${id}`,
        method: "DELETE",
    });
}
