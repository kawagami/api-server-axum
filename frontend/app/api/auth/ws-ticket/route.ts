import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

// 用 httpOnly session cookie 向後端換 30 秒一次性 WS 連線票，
// admin token 不出現在 client 端 JS 與 WS URL
export async function POST() {
    const cookieStore = await cookies();
    const token = cookieStore.get('session')?.value;
    if (!token) return NextResponse.json({ error: 'Missing session' }, { status: 401 });

    const response = await fetch(`${process.env.API_URL}/ws/ticket`, {
        method: 'POST',
        headers: { Authorization: `Bearer ${token}` },
    });

    if (!response.ok) {
        return NextResponse.json({ error: 'Ticket failed' }, { status: response.status });
    }

    const data = await response.json();
    return NextResponse.json(data);
}
