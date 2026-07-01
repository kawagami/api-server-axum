"use client";

import { useState, FormEvent } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Loader2 } from 'lucide-react';
import { startTokenRefresh } from '@/libs/token-refresh';

export default function Login() {
    const router = useRouter();
    const searchParams = useSearchParams();
    const redirectUrl = searchParams.get('redirect') || '/admin';
    const [error, setError] = useState<string | null>(null);
    const [pending, setPending] = useState(false);

    async function handleSubmit(e: FormEvent<HTMLFormElement>) {
        e.preventDefault();
        setError(null);
        setPending(true);

        const form = e.currentTarget;
        const email = (form.elements.namedItem('email') as HTMLInputElement).value;
        const password = (form.elements.namedItem('password') as HTMLInputElement).value;

        try {
            const res = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email, password }),
            });

            if (!res.ok) {
                const data = await res.json();
                setError(data.error || '登入失敗');
                return;
            }

            const { token } = await res.json();
            localStorage.setItem('token', token);
            startTokenRefresh();
            router.push(redirectUrl);
        } catch {
            setError('網路錯誤，請稍後再試');
        } finally {
            setPending(false);
        }
    }

    return (
        <div className="w-full h-[calc(100svh-120px)] overflow-auto flex justify-center items-start">
            <div className="w-full max-w-md p-8 space-y-6 bg-white dark:bg-neutral-800 rounded-lg shadow-md">
                <h2 className="text-2xl font-bold text-center text-neutral-800 dark:text-neutral-100">Login</h2>
                <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                        <label htmlFor="email" className="block text-sm font-medium text-neutral-700 dark:text-neutral-300">Email</label>
                        <input type="email" id="email" name="email" className="w-full px-4 py-2 mt-1 text-neutral-900 dark:text-neutral-100 bg-white dark:bg-neutral-700 border dark:border-neutral-600 rounded-md shadow-sm focus:ring-primary-500 focus:border-primary-500" placeholder="Enter your email" required />
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
