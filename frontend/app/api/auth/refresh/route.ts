import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

// token 只存 httpOnly session cookie，這裡直接讀 cookie 續期，client 端不經手 token
export async function POST() {
    const cookieStore = await cookies();
    const token = cookieStore.get('session')?.value;
    if (!token) return NextResponse.json({ error: 'Missing session' }, { status: 401 });

    const response = await fetch(`${process.env.API_URL}/admin/auth/refresh`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });

    if (!response.ok) {
        return NextResponse.json({ error: 'Refresh failed' }, { status: response.status });
    }

    const newToken = await response.json();

    cookieStore.set('session', newToken, {
        maxAge: 60 * 60,
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
    });

    return NextResponse.json({ ok: true });
}
