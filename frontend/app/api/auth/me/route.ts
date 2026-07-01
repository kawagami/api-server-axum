import { NextRequest, NextResponse } from 'next/server';

export async function GET(req: NextRequest) {
    const auth = req.headers.get('Authorization');
    if (!auth) return NextResponse.json(null, { status: 401 });

    const response = await fetch(`${process.env.API_URL}/admin/auth/me`, {
        headers: { Authorization: auth },
        cache: 'no-store',
    });

    if (!response.ok) {
        return NextResponse.json(null, { status: response.status });
    }

    const user = await response.json();
    return NextResponse.json(user);
}
