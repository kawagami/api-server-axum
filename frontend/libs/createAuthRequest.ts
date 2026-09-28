import { cookies } from "next/headers";
import { clientIpHeaders } from "@/libs/client-ip";

interface RequestOptions {
    url: string;
    method?: string;
    headers?: Record<string, string>;
    body?: BodyInit | null;
}

export function createAuthRequest(cookieKey: string, onUnauthorized: () => Promise<never>) {
    return async function request<T = unknown>({ url, method = 'GET', headers = {}, body = null }: RequestOptions): Promise<T> {
        const cookieStore = await cookies();
        const token = cookieStore.get(cookieKey)?.value;
        // 轉發訪客真實 IP，否則後端限流會把所有人算成 frontend 容器同一個 key
        const ipHeaders = await clientIpHeaders();

        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 30000);

        let response: Response;
        try {
            response = await fetch(url, {
                method,
                headers: {
                    ...(token && { 'Authorization': `Bearer ${token}` }),
                    ...ipHeaders,
                    ...headers,
                },
                body,
                cache: 'no-store',
                signal: controller.signal,
            });
        } finally {
            clearTimeout(timeout);
        }

        // 只有 401（沒登入 / token 失效）才導回登入頁。403 是「已登入但權限不足」
        // （權限放大防護、指派 super_admin…），導去登入頁只會讓人重新登入後又被擋，
        // 看不到原因 —— 照一般錯誤丟出，由呼叫端顯示。
        if (response.status === 401) {
            return await onUnauthorized();
        }

        const text = await response.text();
        const data = text ? (() => { try { return JSON.parse(text); } catch { return null; } })() : null;

        if (!response.ok) {
            const err = new Error(`API ${response.status}: ${response.statusText}`);
            Object.assign(err, { status: response.status, errorData: data });
            throw err;
        }

        return data as T;
    };
}
