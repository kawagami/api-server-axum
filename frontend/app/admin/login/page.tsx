"use client";

import { useActionState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Loader2 } from 'lucide-react';
import { startTokenRefresh } from '@/libs/token-refresh';

type LoginState = { error: string | null; ok: boolean };

export default function Login() {
    const router = useRouter();
    const searchParams = useSearchParams();
    // open-redirect 防護：只收站內相對路徑（同 OAuth callback 的規則）
    const rawRedirect = searchParams.get('redirect');
    const redirectUrl = rawRedirect?.startsWith('/') && !rawRedirect.startsWith('//') ? rawRedirect : '/admin';

    const [state, formAction, pending] = useActionState<LoginState, FormData>(
        async (_prev, formData) => {
            const name = formData.get('name');
            const password = formData.get('password');

            try {
                const res = await fetch('/api/auth/login', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ name, password }),
                });

                if (!res.ok) {
                    const data = await res.json();
                    return { error: data.error || '登入失敗', ok: false };
                }

                return { error: null, ok: true };
            } catch {
                return { error: '網路錯誤，請稍後再試', ok: false };
            }
        },
        { error: null, ok: false }
    );

    const error = state.error;

    useEffect(() => {
        if (state.ok) {
            startTokenRefresh();
            router.push(redirectUrl);
        }
    }, [state.ok, router, redirectUrl]);

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto flex justify-center items-start">
            <div className="w-full max-w-md p-8 space-y-6 bg-white dark:bg-neutral-800 rounded-lg shadow-md">
                <h2 className="text-2xl font-bold text-center text-neutral-800 dark:text-neutral-100">Login</h2>
                <form action={formAction} className="space-y-4">
                    <div>
                        <label htmlFor="name" className="block text-sm font-medium text-neutral-700 dark:text-neutral-300">名稱</label>
                        <input type="text" id="name" name="name" autoComplete="username" className="w-full px-4 py-2 mt-1 text-neutral-900 dark:text-neutral-100 bg-white dark:bg-neutral-700 border dark:border-neutral-600 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500" placeholder="輸入管理員名稱" required />
                    </div>
                    <div>
                        <label htmlFor="password" className="block text-sm font-medium text-neutral-700 dark:text-neutral-300">Password</label>
                        <input type="password" id="password" name="password" className="w-full px-4 py-2 mt-1 text-neutral-900 dark:text-neutral-100 bg-white dark:bg-neutral-700 border dark:border-neutral-600 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500" placeholder="Enter your password" required />
                    </div>
                    {error && <p className="text-sm text-red-500">{error}</p>}
                    <button
                        type="submit"
                        disabled={pending}
                        className={`w-full px-4 py-2 text-white bg-primary-600 rounded-md hover:bg-primary-700 focus:ring-4 focus:ring-primary-500 focus:ring-opacity-50 ${pending ? 'opacity-50 cursor-not-allowed' : ''}`}
                    >
                        {pending ? (
                            <span className="flex items-center justify-center gap-2"><Loader2 className="w-4 h-4 animate-spin" />Logging in...</span>
                        ) : 'Login'}
                    </button>
                </form>
            </div>
        </div>
    );
}
