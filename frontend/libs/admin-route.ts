import "server-only";

import { unstable_rethrow } from "next/navigation";
import { apiErrorStatus, type ApiError } from "@/libs/api-error";

/**
 * 後台「讀取」Route Handler 的共用殼：跑 `load`（通常是 `api/*.ts` 的讀取函式），回 JSON。
 *
 * **為什麼讀取要走 Route Handler 而不是直接在 client 呼叫 Server Action**：Next 在 client 端
 * **一次只送一個 Server Action**（後一個等前一個跑完，見 Next 文件 server-actions「Sequential dispatch」）。
 * 後台有 6 頁在輪詢，輪詢與使用者操作（載入更多、展開軌跡、刪除…）因此互相排隊；
 * Route Handler 是一般的 GET，彼此平行。寫入（mutation）仍走 Server Action。
 *
 * - 401/403：`adminRequest` 會 `redirect()` 到登入頁。那是 Next 內部 error，必須 `unstable_rethrow`
 *   讓它照常變成 307 —— client 端 `libs/admin-queries.ts` 以 `redirect: "manual"` 接住後導頁。
 * - 其他後端錯誤：沿用後端狀態碼，body 帶 `errorData`（與 `ApiError` 同形），client 端還原成 ApiError。
 * - **不做通用代理**：每支 route 只呼叫一個固定的讀取函式、只收白名單參數。通用的「帶 session 打任意路徑」
 *   等於把 adminRequest 開成 endpoint（見 `libs/adminRequest.ts` 檔頭）。
 */
export async function adminJson<T>(load: () => Promise<T>): Promise<Response> {
    try {
        return Response.json(await load(), { headers: { "Cache-Control": "no-store" } });
    } catch (e) {
        unstable_rethrow(e);
        const status = apiErrorStatus(e) ?? 502;
        return Response.json(
            { errorData: (e as ApiError | null)?.errorData ?? null },
            { status, headers: { "Cache-Control": "no-store" } },
        );
    }
}

/** 正整數 query 參數；缺值 / 非法回 undefined（交給讀取函式的預設值） */
export function intParam(params: URLSearchParams, key: string): number | undefined {
    const n = Number(params.get(key));
    return Number.isInteger(n) && n > 0 ? n : undefined;
}

/** 字串 query 參數；空字串視同沒給 */
export function strParam(params: URLSearchParams, key: string): string | undefined {
    return params.get(key) || undefined;
}
