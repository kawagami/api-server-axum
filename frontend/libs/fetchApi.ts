// 預設 10s timeout：避免後端不可達（或 build 階段無 env）時請求吊死
const DEFAULT_TIMEOUT_MS = 10_000;

/**
 * 無認證 fetch 封裝。失敗時丟的 Error 帶 `status` 與 `errorData`
 * （形狀見 `libs/api-error.ts` 的 `ApiError`，與 adminRequest / memberRequest 一致）——
 * 少了這兩個欄位，呼叫端只能拿 message 去比對字串判狀態碼。
 */
export async function fetchApi<T>(url: string, init?: RequestInit): Promise<T> {
    const res = await fetch(url, {
        signal: AbortSignal.timeout(DEFAULT_TIMEOUT_MS),
        ...init,
    });

    if (!res.ok) {
        // 錯誤 body 可能不是 JSON（nginx 的 502 頁、空 body）；解析不出來就當沒有
        let errorData: unknown = null;
        try {
            const text = await res.text();
            errorData = text ? JSON.parse(text) : null;
        } catch {
            errorData = null;
        }
        const err = new Error(`API ${res.status}: ${res.statusText}`);
        Object.assign(err, { status: res.status, errorData });
        throw err;
    }

    return res.json();
}
