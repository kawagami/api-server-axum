import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { clientIpHeaders } from '@/libs/client-ip';

// token 只存 httpOnly session cookie，這裡直接讀 cookie 續期，client 端不經手 token
export async function POST() {
    const cookieStore = await cookies();
    const token = cookieStore.get('session')?.value;
    if (!token) return NextResponse.json({ error: 'Missing session' }, { status: 401 });

    const response = await fetch(`${process.env.API_URL}/admin/auth/refresh`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}`, ...(await clientIpHeaders()) },
    });

    if (!response.ok) {
        return NextResponse.json({ error: 'Refresh failed' }, { status: response.status });
    }

    const newToken = await response.json();

    cookieStore.set('session', newToken, {
        maxAge: 60 * 60,
        httpOnly: true,
        secure: true, // 硬寫：綁 NODE_ENV 的話一旦 env 沒設對，admin JWT 就變成非 Secure cookie
        path: '/',
        sameSite: 'lax',
    });

    return NextResponse.json({ ok: true });
}
