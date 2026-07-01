import { cookies } from 'next/headers'
import { NextRequest, NextResponse } from 'next/server'

export async function GET(
    req: NextRequest,
    { params }: { params: Promise<{ provider: string }> }
) {
    const { provider } = await params
    const res = await fetch(`${process.env.API_URL}/oauth/${provider}`)

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
    return NextResponse.json(data)
}
