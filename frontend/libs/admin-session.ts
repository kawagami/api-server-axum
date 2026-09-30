import type { CookieWriter } from "@/libs/member-session";

/**
 * admin session cookie 的唯一設定來源。密碼登入、passkey 登入、續期三支 Route Handler
 * 都從這裡寫（比照 `libs/member-session.ts`），效期 / 旗標才不會三份各自漂移。
 *
 * 效期對齊後端 admin JWT：1 小時。
 */
export const ADMIN_SESSION_COOKIE = "session";

const SESSION_MAX_AGE = 60 * 60;

/** 後端 `POST /admin/auth`、`/admin/auth/passkeys/login/finish`、`/admin/auth/refresh` 的回應（`AdminTokenResponse`） */
export interface AdminTokens {
    access_token: string;
}

export function setAdminSessionCookie(cookies: CookieWriter, tokens: AdminTokens): void {
    cookies.set(ADMIN_SESSION_COOKIE, tokens.access_token, {
        httpOnly: true,
        // 硬寫：綁 NODE_ENV 的話一旦 env 沒設對，admin JWT 就變成非 Secure cookie
        secure: true,
        sameSite: "lax",
        path: "/",
        maxAge: SESSION_MAX_AGE,
    });
}

/**
 * 登入類端點的錯誤 → 給登入頁看的 `{ error }` + 狀態碼。
 *
 * 429（限流）要原樣透傳：以前一律轉成 500「伺服器錯誤 (429)」，使用者看不出是「試太多次」。
 * 帳密 / passkey 錯誤的文案由呼叫端給（兩者用詞不同）。
 */
export async function loginFailure(response: Response, unauthorizedMessage: string): Promise<{ error: string; status: number }> {
    const status = response.status;
    if (status === 401 || status === 403 || status === 404) {
        return { error: unauthorizedMessage, status: 401 };
    }
    if (status === 429) {
        const body = await response.json().catch(() => null) as { message?: string } | null;
        return { error: body?.message || "嘗試次數過多，請稍後再試", status: 429 };
    }
    if (status === 400 || status === 422) {
        return { error: "請求格式錯誤", status: 400 };
    }
    return { error: `伺服器錯誤 (${status})`, status: 500 };
}
