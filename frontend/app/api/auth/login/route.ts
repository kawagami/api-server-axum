import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST(req: NextRequest) {
    const body = await req.json();

    const response = await fetch(`${process.env.API_URL}/admin/auth`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
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
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
    });

    return NextResponse.json({ token });
}
