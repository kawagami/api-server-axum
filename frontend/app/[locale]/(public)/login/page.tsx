'use client'

import { Suspense } from 'react'
import { useSearchParams } from 'next/navigation'
import { useTranslations } from 'next-intl'

async function handleLogin(provider: string, redirectTo: string | null) {
    const qs = redirectTo ? `?redirect=${encodeURIComponent(redirectTo)}` : ''
    const res = await fetch(`/api/auth/${provider}${qs}`)
    if (!res.ok) return
    const { url } = await res.json()
    window.location.href = url
}

function LoginContent() {
    const searchParams = useSearchParams()
    const t = useTranslations('Login')
    const error = searchParams.get('error')
    const redirectTo = searchParams.get('redirect')

    const errorMsg = error
        ? (error === 'oauth_failed' ? t('errorOauthFailed') : error === 'oauth_denied' ? t('errorOauthDenied') : t('errorDefault'))
        : null

    return (
        <div className="flex flex-col items-center justify-center gap-4 pt-16">
            <h1 className="text-2xl font-bold">{t('title')}</h1>
            {errorMsg && (
                <p className="text-red-500 dark:text-red-400 text-sm">{errorMsg}</p>
            )}
            <div className="flex flex-col gap-3 w-64">
                <button
                    onClick={() => handleLogin('google', redirectTo)}
                    className="flex items-center justify-center gap-2 px-4 py-2 rounded-lg border border-neutral-300 bg-white text-neutral-700 hover:bg-neutral-50 dark:bg-neutral-800 dark:text-white dark:border-neutral-600 dark:hover:bg-neutral-700 transition-colors"
                >
                    {t('googleLogin')}
                </button>
                <button
                    disabled
                    className="flex items-center justify-center gap-2 px-4 py-2 rounded-lg border border-neutral-200 bg-neutral-100 text-neutral-400 dark:bg-neutral-800 dark:text-neutral-500 dark:border-neutral-700 cursor-not-allowed"
                >
                    {t('githubLogin')} <span className="text-xs">（{t('comingSoon')}）</span>
                </button>
                <button
                    disabled
                    className="flex items-center justify-center gap-2 px-4 py-2 rounded-lg border border-neutral-200 bg-neutral-100 text-neutral-400 dark:bg-neutral-800 dark:text-neutral-500 dark:border-neutral-700 cursor-not-allowed"
                >
                    {t('lineLogin')} <span className="text-xs">（{t('comingSoon')}）</span>
                </button>
            </div>
        </div>
    )
}

export default function LoginPage() {
    return (
        <Suspense>
            <LoginContent />
        </Suspense>
    )
}
