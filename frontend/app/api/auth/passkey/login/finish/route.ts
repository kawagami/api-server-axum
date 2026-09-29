import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';
import { clientIpHeaders } from '@/libs/client-ip';
import { loginFailure, setAdminSessionCookie, type AdminTokens } from '@/libs/admin-session';

// 後端回傳與密碼登入同形的 `{ access_token }`，cookie 寫法比照 /api/auth/login
export async function POST(req: NextRequest) {
    const body = await req.json();

    const response = await fetch(`${process.env.API_URL}/admin/auth/passkeys/login/finish`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await clientIpHeaders()) },
        body: JSON.stringify(body),
    });

    if (!response.ok) {
        // 401 = 挑戰過期/驗證失敗 —— 前端據此靜默重試一次
        const { error, status } = await loginFailure(response, 'Passkey 驗證失敗');
        return NextResponse.json({ error }, { status });
    }

    setAdminSessionCookie(await cookies(), (await response.json()) as AdminTokens);
    return NextResponse.json({ ok: true });
}
