import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

// 後端回傳與密碼登入同形的 JWT，cookie 寫法比照 /api/auth/login
export async function POST(req: NextRequest) {
    const body = await req.json();

    const response = await fetch(`${process.env.API_URL}/admin/auth/passkeys/login/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });

    if (!response.ok) {
        const status = response.status;
        if (status === 401) {
            // 挑戰過期/驗證失敗——前端據此靜默重試一次
            return NextResponse.json({ error: 'Passkey 驗證失敗' }, { status: 401 });
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

    return NextResponse.json({ ok: true });
}
