/**
 * 會員 session cookie 的唯一設定來源。OAuth callback（首次發 token）與 proxy（續期）
 * 都從這裡寫，兩邊的效期 / 旗標才不會漂移。
 *
 * 效期對齊後端 `services/oauth.rs`：access 1 小時、refresh 30 天。
 * access_token cookie 與 JWT 同時過期，所以「cookie 不見了」就等於「該續期了」。
 */
export const MEMBER_ACCESS_COOKIE = "access_token";
export const MEMBER_REFRESH_COOKIE = "refresh_token";

const ACCESS_MAX_AGE = 60 * 60;
const REFRESH_MAX_AGE = 60 * 60 * 24 * 30;

export interface MemberTokens {
    access_token: string;
    refresh_token: string;
}

/** `cookies()`（Route Handler）與 `NextResponse.cookies` 都滿足這個形狀 */
interface CookieWriter {
    set(name: string, value: string, options: {
        httpOnly: boolean;
        secure: boolean;
        sameSite: "lax";
        path: string;
        maxAge: number;
    }): unknown;
}

export function setMemberSessionCookies(cookies: CookieWriter, tokens: MemberTokens): void {
    const base = { httpOnly: true, secure: true, sameSite: "lax" as const, path: "/" };
    cookies.set(MEMBER_ACCESS_COOKIE, tokens.access_token, { ...base, maxAge: ACCESS_MAX_AGE });
    cookies.set(MEMBER_REFRESH_COOKIE, tokens.refresh_token, { ...base, maxAge: REFRESH_MAX_AGE });
}
