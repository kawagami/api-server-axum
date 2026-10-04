import { jwtVerify } from "jose";
import createMiddleware from 'next-intl/middleware';
import { NextRequest, NextResponse } from "next/server";
import { routing } from './i18n/routing';
import {
    MEMBER_ACCESS_COOKIE,
    MEMBER_REFRESH_COOKIE,
    setMemberSessionCookies,
    type MemberTokens,
} from './libs/member-session';

const intlMiddleware = createMiddleware(routing);

export const config = {
    // opengraph-image 沒有副檔名，不排除的話會被 intl middleware 加上 locale prefix 而 404
    matcher: ['/((?!_next|api|auth|opengraph-image|.*\\..*).*)'],
};

/** access_token 離過期不到這麼久就先續，免得 render 到一半過期 */
const REFRESH_LEEWAY_SEC = 60;

function jwtSecret(): Uint8Array {
    return new TextEncoder().encode(process.env.JWT_SECRET);
}

/** 簽章有效、是會員 token、且離過期超過 leeway */
async function isFreshMemberToken(token: string | undefined): Promise<boolean> {
    if (!token) return false;
    try {
        const { payload } = await jwtVerify(token, jwtSecret());
        return payload.role === 'member'
            && typeof payload.exp === 'number'
            && payload.exp - Date.now() / 1000 > REFRESH_LEEWAY_SEC;
    } catch {
        return false;
    }
}

/**
 * access_token 沒了（cookie 與 JWT 同為 1 小時）或快過期、且手上有 refresh_token 時，
 * 向後端 `POST /oauth/refresh` 換一組新的。拿不到就回 null，這次請求照「未登入」處理。
 *
 * - 失敗時**不刪** refresh_token cookie：後端每次續期都會輪換 jti，兩個請求同時拿同一張
 *   refresh token 來換時，後到的那個必然失敗；這時刪 cookie 會把先到那個剛寫下的新 token
 *   一起蓋掉。真正失效的 refresh token 由它自己的 30 天效期或下次登入覆寫收掉。
 * - refresh token 本身已過期 / 簽章不對就不打後端，省一趟必敗的請求。
 * - router prefetch 不續期：頁面載入時會一次平行發多個 prefetch，全都帶同一張 refresh
 *   token 去換，只有一個會成功，其餘反而撞上上面那種輪換競態。
 */
async function refreshMemberTokens(req: NextRequest): Promise<MemberTokens | null> {
    if (req.headers.has('next-router-prefetch')) return null;

    const refreshToken = req.cookies.get(MEMBER_REFRESH_COOKIE)?.value;
    if (!refreshToken) return null;
    if (await isFreshMemberToken(req.cookies.get(MEMBER_ACCESS_COOKIE)?.value)) return null;

    try {
        await jwtVerify(refreshToken, jwtSecret());
    } catch {
        return null;
    }

    try {
        // 同 libs/client-ip.ts：轉發訪客真實 IP，否則後端限流把所有人算成 frontend 容器
        const ip = req.headers.get('CF-Connecting-IP');
        const res = await fetch(`${process.env.API_URL}/oauth/refresh`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', ...(ip ? { 'CF-Connecting-IP': ip } : {}) },
            body: JSON.stringify({ refresh_token: refreshToken }),
            cache: 'no-store',
            signal: AbortSignal.timeout(5_000),
        });
        if (!res.ok) return null;
        const tokens = (await res.json()) as Partial<MemberTokens>;
        return tokens.access_token && tokens.refresh_token
            ? { access_token: tokens.access_token, refresh_token: tokens.refresh_token }
            : null;
    } catch {
        return null;
    }
}

const OVERRIDE_HEADERS = 'x-middleware-override-headers';
const REQUEST_HEADER_PREFIX = 'x-middleware-request-';

/**
 * 讓**這一次**請求的 Server Component / Server Action 就讀到新 token。
 *
 * `cookies()` 讀的是 request header；只寫 Set-Cookie 的話要等瀏覽器下一個請求才生效，
 * 這次 render 會以未登入身分跑（會員頁被 memberRequest 的 401 導去登入頁）。
 * 官方做法是 `NextResponse.next({ request: { headers } })`，但 response 由 intl middleware
 * 產生，所以把那組覆寫 header 算出來併進去 —— 與 intl 自己設的覆寫取聯集，不蓋掉它的。
 *
 * ⚠️ 兩個 header 名稱是 Next 內部實作（非公開 API），**每次升 Next 都要驗**，步驟見
 * ARCHITECTURE.md「認證」的會員續期一節。改名時不會報錯，只會讓過期會員被導去登入頁。
 */
function forwardRequestCookies(res: NextResponse, req: NextRequest): void {
    const carrier = NextResponse.next({ request: { headers: req.headers } });
    const keys = new Set(
        [res.headers.get(OVERRIDE_HEADERS), carrier.headers.get(OVERRIDE_HEADERS)]
            .flatMap(v => v?.split(',') ?? [])
            .filter(Boolean),
    );
    carrier.headers.forEach((value, key) => {
        if (key.startsWith(REQUEST_HEADER_PREFIX) && !res.headers.has(key)) res.headers.set(key, value);
    });
    keys.add('cookie');
    res.headers.set(`${REQUEST_HEADER_PREFIX}cookie`, req.headers.get('cookie') ?? '');
    res.headers.set(OVERRIDE_HEADERS, [...keys].join(','));
}

export default async function proxy(req: NextRequest) {
    const path = req.nextUrl.pathname;

    // Admin routes — auth check, skip intl
    if (path.startsWith('/admin')) {
        if (!path.startsWith('/admin/login')) {
            const value = req.cookies.get('session')?.value;
            const loginUrl = new URL('/admin/login', req.url);
            loginUrl.searchParams.set('redirect', path + req.nextUrl.search);

            if (!value) return NextResponse.redirect(loginUrl);

            try {
                const { payload } = await jwtVerify(value, jwtSecret());
                // 也要驗 role：admin 與 member 的 token 用同一把 secret 簽，
                // 只驗簽章的話把 member 的 access_token 塞進 session cookie 就能進
                // /admin/* 的頁面外殼。後端 authorize_and_load 會擋成 401（無資料外洩），
                // 但前端不該比後端寬鬆。
                if (payload.role !== 'admin') return NextResponse.redirect(loginUrl);
            } catch {
                return NextResponse.redirect(loginUrl);
            }
        }
        return NextResponse.next();
    }

    // 會員 token 續期：必須在下面的會員頁檢查之前，否則過期的人會先被導去登入頁
    const tokens = await refreshMemberTokens(req);
    if (tokens) {
        req.cookies.set(MEMBER_ACCESS_COOKIE, tokens.access_token);
        req.cookies.set(MEMBER_REFRESH_COOKIE, tokens.refresh_token);
    }

    // Member-only routes — check access_token
    // 功能的 settings 子頁由所屬 prefix 涵蓋，不另列
    const memberPaths = ['/dashboard', '/profile', '/portfolio', '/food-log'];
    const isMemberRoute = routing.locales.some(locale =>
        memberPaths.some(p => path === `/${locale}${p}` || path.startsWith(`/${locale}${p}/`))
    );

    if (isMemberRoute) {
        const accessToken = req.cookies.get(MEMBER_ACCESS_COOKIE)?.value;
        if (!accessToken) {
            const locale = routing.locales.find(l => path.startsWith(`/${l}/`) || path === `/${l}`) ?? routing.defaultLocale;
            const loginUrl = new URL(`/${locale}/login`, req.url);
            loginUrl.searchParams.set('redirect', path + req.nextUrl.search);
            return NextResponse.redirect(loginUrl);
        }
    }

    // Apply intl routing for all public routes
    const res = intlMiddleware(req);
    if (tokens) {
        setMemberSessionCookies(res.cookies, tokens);
        // intl 回 redirect（例如補 locale 前綴）時，瀏覽器跟過去自然帶新 cookie，不必轉發
        if (!res.headers.has('location')) forwardRequestCookies(res, req);
    }
    return res;
}
