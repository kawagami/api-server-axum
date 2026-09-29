import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { clientIpHeaders } from '@/libs/client-ip';
import { loginFailure, setAdminSessionCookie, type AdminTokens } from '@/libs/admin-session';

export async function POST(req: NextRequest) {
    const body = await req.json();

    const response = await fetch(`${process.env.API_URL}/admin/auth`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await clientIpHeaders()) },
        body: JSON.stringify(body),
    });

    if (!response.ok) {
        const { error, status } = await loginFailure(response, '帳號或密碼錯誤');
        return NextResponse.json({ error }, { status });
    }

    setAdminSessionCookie(await cookies(), (await response.json()) as AdminTokens);
    return NextResponse.json({ ok: true });
}
