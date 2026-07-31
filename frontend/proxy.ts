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
                await jwtVerify(value, secret);
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
