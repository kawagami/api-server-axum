import { jwtVerify } from "jose";
import createMiddleware from 'next-intl/middleware';
import { NextRequest, NextResponse } from "next/server";
import { routing } from './i18n/routing';

const intlMiddleware = createMiddleware(routing);

export const config = {
    // opengraph-image 沒有副檔名，不排除的話會被 intl middleware 加上 locale prefix 而 404
    matcher: ['/((?!_next|api|auth|opengraph-image|.*\\..*).*)'],
};

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
                const secret = new TextEncoder().encode(process.env.JWT_SECRET);
                const { payload } = await jwtVerify(value, secret);
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

    // Member-only routes — check access_token
    // 各功能的 settings 子頁（/invoices/settings、/lotto/settings）由所屬 prefix 涵蓋，不另列
    const memberPaths = ['/dashboard', '/profile', '/portfolio', '/ledger', '/invoices', '/lotto'];
    const isMemberRoute = routing.locales.some(locale =>
        memberPaths.some(p => path === `/${locale}${p}` || path.startsWith(`/${locale}${p}/`))
    );

    if (isMemberRoute) {
        const accessToken = req.cookies.get('access_token')?.value;
        if (!accessToken) {
            const locale = routing.locales.find(l => path.startsWith(`/${l}/`) || path === `/${l}`) ?? routing.defaultLocale;
            const loginUrl = new URL(`/${locale}/login`, req.url);
            loginUrl.searchParams.set('redirect', path + req.nextUrl.search);
            return NextResponse.redirect(loginUrl);
        }
    }

    // Apply intl routing for all public routes
    return intlMiddleware(req);
}
