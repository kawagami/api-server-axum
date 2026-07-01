import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

export async function POST(req: NextRequest) {
    const auth = req.headers.get('Authorization');
    if (!auth) return NextResponse.json({ error: 'Missing token' }, { status: 401 });

    const response = await fetch(`${process.env.API_URL}/admin/auth/refresh`, {
        method: 'POST',
        headers: { Authorization: auth },
    });

    if (!response.ok) {
        return NextResponse.json({ error: 'Refresh failed' }, { status: response.status });
    }

    const newToken = await response.json();

    const cookieStore = await cookies();
    cookieStore.set('session', newToken, {
        maxAge: 60 * 60,
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
    });

    return NextResponse.json({ token: newToken });
}
