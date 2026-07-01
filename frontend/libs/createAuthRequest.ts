import { cookies } from "next/headers";

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

        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), 30000);

        let response: Response;
        try {
            response = await fetch(url, {
                method,
                headers: {
                    ...(token && { 'Authorization': `Bearer ${token}` }),
                    ...headers,
                },
                body,
                cache: 'no-store',
                signal: controller.signal,
            });
        } finally {
            clearTimeout(timeout);
        }

        if (response.status === 401 || response.status === 403) {
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
