import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { NextRequest } from 'next/server'
import { clientIpHeaders } from '@/libs/client-ip'

// 同 /api/auth/[provider]：擋掉用 %2F 編碼把 provider 變成路徑片段的代理用法
const PROVIDERS = ['google', 'github', 'line'] as const

export async function GET(
    request: NextRequest,
    { params }: { params: Promise<{ provider: string }> }
) {
    const { provider } = await params
    if (!PROVIDERS.includes(provider as (typeof PROVIDERS)[number])) {
        redirect('/login?error=oauth_failed')
    }
    const { searchParams } = request.nextUrl

    const code = searchParams.get('code')
    const state = searchParams.get('state')
    const error = searchParams.get('error')

    if (error || !code || !state) {
        redirect('/login?error=oauth_denied')
    }

    // state 必須與發起本次登入時寫下的 cookie 相符，否則這是別人的授權流程被塞進來
    // （login CSRF）。cookie 由 /api/auth/[provider] 寫入，httpOnly 故攻擊者無法設定。
    // 用後即刪，避免殘留讓下一次 callback 意外通過。
    const cookieStore = await cookies()
    const expectedState = cookieStore.get('oauth_state')?.value
    cookieStore.delete('oauth_state')
    if (!expectedState || expectedState !== state) {
        redirect('/login?error=oauth_denied')
    }

    const res = await fetch(`${process.env.API_URL}/oauth/${provider}/exchange`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await clientIpHeaders()) },
        body: JSON.stringify({ code, state }),
    })

    if (!res.ok) {
        redirect('/login?error=oauth_failed')
    }

    const { access_token, refresh_token } = await res.json()

    cookieStore.set('access_token', access_token, {
        httpOnly: true,
        secure: true,
        sameSite: 'lax',
        maxAge: 60 * 60,
    })
    cookieStore.set('refresh_token', refresh_token, {
        httpOnly: true,
        secure: true,
        sameSite: 'lax',
        maxAge: 60 * 60 * 24 * 30,
    })

    // Return to the page the member originally tried to reach, if any.
    const dest = cookieStore.get('post_login_redirect')?.value
    cookieStore.delete('post_login_redirect')

    redirect(dest && dest.startsWith('/') && !dest.startsWith('//') ? dest : '/')
}
