import { unstable_rethrow } from "next/navigation";

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

/**
 * Server Action 的失敗結果。
 *
 * Server Action **丟出**的錯誤在 production 會被 Next 清掉 `status` / `errorData`
 * （client 只拿到一個通用訊息加 digest），於是 client 端的 `e.status === 409` 這類分流
 * 只在 dev 有效。要讓 client 依狀態碼顯示訊息，就得把它放在**回傳值**裡帶過邊界。
 */
export interface ActionFailure {
    ok: false;
    /** 網路中斷 / 逾時為 undefined */
    status?: number;
    message?: string;
}

export type ActionResult<T = void> = { ok: true; data: T } | ActionFailure;

/**
 * 把 catch 到的錯誤轉成 `ActionFailure`。只能在 server 端（Server Action 內）呼叫。
 * `adminRequest` / `memberRequest` 在 401 丟的是 Next 的 redirect，必須重丟，吞掉導頁就失效。
 */
export function toActionFailure(e: unknown): ActionFailure {
    unstable_rethrow(e);
    return { ok: false, status: apiErrorStatus(e), message: (e as ApiError | null)?.errorData?.message };
}

/**
 * Server Action 的標準包裝：`fn` 成功 → `{ ok: true, data }`，丟錯 → `toActionFailure`
 * （401 的 redirect 照常重丟）。client 直接呼叫的 Server Action 一律用它，不要自己寫 try/catch。
 */
export async function runAction<T>(fn: () => Promise<T>): Promise<ActionResult<T>> {
    try {
        return { ok: true, data: await fn() };
    } catch (e) {
        return toActionFailure(e);
    }
}
