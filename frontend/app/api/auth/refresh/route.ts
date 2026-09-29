import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { clientIpHeaders } from '@/libs/client-ip';
import { ADMIN_SESSION_COOKIE, setAdminSessionCookie, type AdminTokens } from '@/libs/admin-session';

// token 只存 httpOnly session cookie，這裡直接讀 cookie 續期，client 端不經手 token
export async function POST() {
    const cookieStore = await cookies();
    const token = cookieStore.get(ADMIN_SESSION_COOKIE)?.value;
    if (!token) return NextResponse.json({ error: 'Missing session' }, { status: 401 });

    const response = await fetch(`${process.env.API_URL}/admin/auth/refresh`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}`, ...(await clientIpHeaders()) },
    });

    if (!response.ok) {
        return NextResponse.json({ error: 'Refresh failed' }, { status: response.status });
    }

    setAdminSessionCookie(cookieStore, (await response.json()) as AdminTokens);

    return NextResponse.json({ ok: true });
}
