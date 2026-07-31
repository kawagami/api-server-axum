import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { clientIpHeaders } from '@/libs/client-ip';

export async function POST(req: NextRequest) {
    const body = await req.json();

    const response = await fetch(`${process.env.API_URL}/admin/auth`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await clientIpHeaders()) },
        body: JSON.stringify(body),
    });

    if (!response.ok) {
        const status = response.status;
        if (status === 401 || status === 403 || status === 404) {
            return NextResponse.json({ error: '帳號或密碼錯誤' }, { status: 401 });
        }
        return NextResponse.json({ error: `伺服器錯誤 (${status})` }, { status: 500 });
    }

    const token = await response.json();

    const cookieStore = await cookies();
    cookieStore.set('session', token, {
        maxAge: 60 * 60,
        httpOnly: true,
        secure: true, // 硬寫：綁 NODE_ENV 的話一旦 env 沒設對，admin JWT 就變成非 Secure cookie
        path: '/',
        sameSite: 'lax',
    });

    return NextResponse.json({ ok: true });
}
