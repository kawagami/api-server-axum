// 預設 10s timeout：避免後端不可達（或 build 階段無 env）時請求吊死
const DEFAULT_TIMEOUT_MS = 10_000;

export async function fetchApi<T>(url: string, init?: RequestInit): Promise<T> {
    const res = await fetch(url, {
        signal: AbortSignal.timeout(DEFAULT_TIMEOUT_MS),
        ...init,
    });
    if (!res.ok) throw new Error(`API ${res.status}: ${res.statusText}`);
    return res.json();
}
