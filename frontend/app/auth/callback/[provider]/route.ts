import { cookies } from 'next/headers'
import { redirect } from 'next/navigation'
import { NextRequest } from 'next/server'

export async function GET(
    request: NextRequest,
    { params }: { params: Promise<{ provider: string }> }
) {
    const { provider } = await params
    const { searchParams } = request.nextUrl

    const code = searchParams.get('code')
    const state = searchParams.get('state')
    const error = searchParams.get('error')

    if (error || !code || !state) {
        redirect('/login?error=oauth_denied')
    }

    const res = await fetch(`${process.env.API_URL}/oauth/${provider}/exchange`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code, state }),
    })

    if (!res.ok) {
        redirect('/login?error=oauth_failed')
    }

    const { access_token, refresh_token } = await res.json()
    const cookieStore = await cookies()

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
