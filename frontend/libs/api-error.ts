/**
 * 後端錯誤回應在前端的形狀。
 *
 * `createAuthRequest`（adminRequest / memberRequest）與 `fetchApi` 失敗時都丟這個：
 * message 是 `API {status}: {statusText}`，另掛 `status`（number）與 `errorData`（parse 過的 body）。
 *
 * 存在的理由：型別沒匯出時，每個 catch 區塊都得自己 inline cast
 * `err as Error & { status?: number; ... }`，而漏掉的地方只好退化成比對訊息字串
 * （`msg.includes("429")`）—— 那種寫法在錯誤訊息改文案的當下就靜默失效。
 */
export interface ApiError extends Error {
    status?: number;
    errorData?: { message?: string; code?: string } | null;
}

/** HTTP 狀態碼；不是 API 錯誤（網路中斷、逾時）回 undefined */
export function apiErrorStatus(e: unknown): number | undefined {
    const status = (e as ApiError | null)?.status;
    return typeof status === "number" ? status : undefined;
}

/** 後端給的錯誤訊息，沒有就用呼叫端的 fallback（不要把 `API 500: …` 這種原文露給使用者） */
export function apiErrorMessage(e: unknown, fallback: string): string {
    return (e as ApiError | null)?.errorData?.message || fallback;
}
