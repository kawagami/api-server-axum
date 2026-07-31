import { cookies } from 'next/headers'
import { NextRequest, NextResponse } from 'next/server'
import { clientIpHeaders } from '@/libs/client-ip'

export async function GET(
    req: NextRequest,
    { params }: { params: Promise<{ provider: string }> }
) {
    const { provider } = await params
    const res = await fetch(`${process.env.API_URL}/oauth/${provider}`, {
        headers: await clientIpHeaders(),
    })

    if (!res.ok) {
        return NextResponse.json({ error: 'failed to get auth url' }, { status: res.status })
    }

    // Stash the intended destination so the OAuth callback can return there.
    // Guard against open redirects: only allow site-relative paths.
    const redirectTo = req.nextUrl.searchParams.get('redirect')
    if (redirectTo && redirectTo.startsWith('/') && !redirectTo.startsWith('//')) {
        const cookieStore = await cookies()
        cookieStore.set('post_login_redirect', redirectTo, {
            httpOnly: true,
            secure: true,
            sameSite: 'lax',
            maxAge: 60 * 10,
        })
    }

    const data = await res.json()

    // 把後端產生的 state 綁在這個瀏覽器上（login CSRF 防護）。
    //
    // 後端只把 state 存進 Redis 驗「這個 state 存在過」，不含任何瀏覽器識別 ——
    // 攻擊者可以自己走完授權、攔下 code 不放行，再誘導受害者開
    // /auth/callback/google?code=<攻擊者的>&state=<配對的>，受害者的瀏覽器就被寫入
    // **攻擊者帳號**的 token，之後記的發票／記帳／樂透選號全進攻擊者帳號。
    // callback 端會比對這個 cookie 與 query 的 state，不符就不 exchange。
    // 後端的 Redis 一次性消費保留當第二層（防重放）。
    const state = new URL(data.url).searchParams.get('state')
    if (state) {
        const cookieStore = await cookies()
        cookieStore.set('oauth_state', state, {
            httpOnly: true,
            secure: true,
            sameSite: 'lax',
            path: '/',
            maxAge: 300, // 與後端 Redis oauth:state 的 TTL 對齊
        })
    }

    return NextResponse.json(data)
}
