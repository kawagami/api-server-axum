import { NextResponse } from 'next/server';

// 登入前無 session，同源代理後端 passkey 挑戰（瀏覽器不直打後端）
export async function POST() {
    const response = await fetch(`${process.env.API_URL}/admin/auth/passkeys/login/begin`, {
        method: 'POST',
    });

    if (!response.ok) {
        const status = response.status === 429 ? 429 : 500;
        return NextResponse.json({ error: `伺服器錯誤 (${response.status})` }, { status });
    }

    return NextResponse.json(await response.json());
}
